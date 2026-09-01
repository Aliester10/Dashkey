/// UDP discovery — mencari Host DashKey di jaringan lokal tanpa scan QR.
///
/// Controller mengirim broadcast `dashkey_discover_v1` ke port discovery;
/// setiap Host yang aktif membalas `dashkey_hello` berisi host, port,
/// nama, versi, dan pair_token segar (siap dipakai pairing langsung).
library;

import 'dart:convert';
import 'dart:io';

/// Host DashKey yang ditemukan di jaringan.
class DiscoveredHost {
  const DiscoveredHost({
    required this.host,
    required this.port,
    required this.hostName,
    required this.version,
    required this.pairToken,
  });

  final String host;
  final int port;
  final String hostName;
  final String version;
  final String pairToken;

  factory DiscoveredHost.fromJson(Map<String, dynamic> json) => DiscoveredHost(
        host: json['host'] as String? ?? '',
        port: (json['port'] as num?)?.toInt() ?? 48484,
        hostName: json['host_name'] as String? ?? 'DashKey Host',
        version: json['version'] as String? ?? '',
        pairToken: json['pair_token'] as String? ?? '',
      );

  bool get isValid => host.isNotEmpty && pairToken.isNotEmpty;
}

/// Proses discovery via UDP broadcast.
class HostDiscovery {
  HostDiscovery._();

  /// Magic request yang dikirim Controller ke jaringan.
  static const magic = 'dashkey_discover_v1';

  /// Port UDP tempat Host mendengarkan request discovery.
  static const discoveryPort = 48485;

  /// Kirim broadcast, kumpulkan jawaban host selama [timeout].
  /// Jawaban di-dedupe per host:port.
  static Future<List<DiscoveredHost>> discover({
    Duration timeout = const Duration(seconds: 2),
  }) async {
    final socket = await RawDatagramSocket.bind(InternetAddress.anyIPv4, 0);
    socket.broadcastEnabled = true;
    final results = <DiscoveredHost>[];

    socket.listen((event) {
      if (event != RawSocketEvent.read) return;
      final datagram = socket.receive();
      if (datagram == null) return;
      try {
        final json =
            jsonDecode(utf8.decode(datagram.data)) as Map<String, dynamic>;
        if (json['type'] != 'dashkey_hello') return;
        final host = DiscoveredHost.fromJson(json);
        if (!host.isValid) return;
        if (!results.any((h) => h.host == host.host && h.port == host.port)) {
          results.add(host);
        }
      } catch (_) {
        // Abaikan paket yang bukan hello DashKey.
      }
    });

    try {
      socket.send(
        utf8.encode(magic),
        InternetAddress('255.255.255.255'),
        discoveryPort,
      );
    } catch (_) {
      // Broadcast bisa gagal di jaringan tertentu — hasil kosong.
    }

    await Future<void>.delayed(timeout);
    socket.close();
    return results;
  }
}