/// ClockPage — halaman khusus Clock Mode (PRD §5, §21, §23, §25).
///
/// - Fullscreen: status bar & navigation bar disembunyikan (immersive).
/// - Layout murni: jam di tengah, tanggal di bawah — tanpa elemen aplikasi.
/// - Responsive: ukuran kartu dihitung dari available space & orientasi.
/// - Tap pada layar → keluar dari Clock Mode.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../widgets/flip_clock.dart';

class ClockPage extends StatefulWidget {
  const ClockPage({super.key});

  @override
  State<ClockPage> createState() => _ClockPageState();
}

class _ClockPageState extends State<ClockPage> {
  @override
  void initState() {
    super.initState();
    // PRD §21: fullscreen — sembunyikan system UI.
    SystemChrome.setEnabledSystemUIMode(SystemUiMode.immersiveSticky);
  }

  @override
  void dispose() {
    // Kembalikan system UI normal saat keluar.
    SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0F1115),
      body: GestureDetector(
        behavior: HitTestBehavior.opaque,
        child: SafeArea(
          child: LayoutBuilder(
            builder: (context, constraints) {
              final isPortrait =
                  MediaQuery.of(context).orientation == Orientation.portrait;
              return _buildClock(
                constraints.maxWidth,
                constraints.maxHeight,
                isPortrait,
              );
            },
          ),
        ),
      ),
    );
  }

  /// Hitung ukuran kartu & layout (PRD §17–§18: responsive, no overflow).
  Widget _buildClock(double availW, double availH, bool isPortrait) {
    // Ukuran dasar kartu per orientasi (PRD §18).
    var cardHeight = isPortrait
        ? (availH * 0.17).clamp(96.0, 210.0)
        : (availH * 0.48).clamp(160.0, 300.0);
    var cardWidth = cardHeight * 0.58;

    // Estimasi lebar total: 6 kartu + 2 separator + 3 gap.
    final gap = cardWidth * 0.10;
    final separatorW = cardHeight * 0.18;
    final totalW = cardWidth * 6 + separatorW * 2 + gap * 3;

    // Skala turun bila melebihi lebar layar (anti overflow).
    if (totalW > availW) {
      final scale = availW / totalW;
      cardWidth *= scale;
      cardHeight *= scale;
    }

    // Tinggi total: jam + date; skala bila melebihi tinggi.
    final totalH = cardHeight + 32 + 40;
    if (totalH > availH) {
      final scale = availH / totalH;
      cardWidth *= scale;
      cardHeight *= scale;
    }

    return Center(
      child: SingleChildScrollView(
        child: FlipClock(cardWidth: cardWidth, cardHeight: cardHeight),
      ),
    );
  }
}
