/// Host Discovery Controller — state pencarian otomatis host DashKey.
library;

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../services/host_discovery.dart';

/// State hasil discovery.
class HostDiscoveryState {
  const HostDiscoveryState({
    this.scanning = false,
    this.hosts = const [],
    this.error,
  });

  final bool scanning;
  final List<DiscoveredHost> hosts;
  final String? error;

  HostDiscoveryState copyWith({
    bool? scanning,
    List<DiscoveredHost>? hosts,
    String? error,
  }) =>
      HostDiscoveryState(
        scanning: scanning ?? this.scanning,
        hosts: hosts ?? this.hosts,
        error: error ?? this.error,
      );
}

final hostDiscoveryProvider = NotifierProvider<HostDiscoveryController, HostDiscoveryState>(
  HostDiscoveryController.new,
);

class HostDiscoveryController extends Notifier<HostDiscoveryState> {
  @override
  HostDiscoveryState build() => const HostDiscoveryState();

  /// Mulai pencarian: broadcast UDP + tunggu jawaban host ~2 detik.
  Future<void> discover() async {
    if (state.scanning) return;
    state = state.copyWith(scanning: true, error: null);
    try {
      final hosts = await HostDiscovery.discover();
      state = state.copyWith(scanning: false, hosts: hosts);
    } catch (e) {
      state = state.copyWith(scanning: false, error: '$e');
    }
  }

  /// Hapus hasil agar pencarian ulang dari nol.
  void clear() => state = const HostDiscoveryState();
}