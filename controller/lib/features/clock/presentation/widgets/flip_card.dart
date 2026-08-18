/// FlipCard — kartu flip mekanik neumorphic (PRD §9, §16, §25).
library;

import 'package:flutter/material.dart';

class FlipCard extends StatelessWidget {
  const FlipCard({
    super.key,
    required this.width,
    required this.height,
    required this.oldValue,
    required this.newValue,
    required this.progress,
  });

  final double width;
  final double height;
  final int oldValue;
  final int newValue;
  final double progress; // 0.0 to 1.0

  @override
  Widget build(BuildContext context) {
    final double upperT = (progress * 2.0).clamp(0.0, 1.0);
    final double lowerT = ((progress - 0.5) * 2.0).clamp(0.0, 1.0);

    return SizedBox(
      width: width,
      height: height,
      child: Stack(
        children: [
          // 1. Static Top (shows new value)
          Positioned(
            top: 0, left: 0, right: 0,
            child: _HalfCard(
              width: width,
              height: height,
              value: newValue,
              isTop: true,
            ),
          ),
          // 2. Static Bottom (shows old value)
          Positioned(
            bottom: 0, left: 0, right: 0,
            child: _HalfCard(
              width: width,
              height: height,
              value: oldValue,
              isTop: false,
            ),
          ),
          // 3. Flip Bottom (animates from -90 to 0, shows new value)
          if (progress >= 0.5)
            Positioned(
              bottom: 0, left: 0, right: 0,
              child: Transform(
                alignment: Alignment.topCenter,
                transform: Matrix4.identity()
                  ..setEntry(3, 2, 0.0015)
                  ..rotateX(1.5708 * (1.0 - lowerT)), // 90 degrees to 0
                child: _HalfCard(
                  width: width,
                  height: height,
                  value: newValue,
                  isTop: false,
                ),
              ),
            ),
          // 4. Flip Top (animates from 0 to -90, shows old value)
          if (progress < 0.5)
            Positioned(
              top: 0, left: 0, right: 0,
              child: Transform(
                alignment: Alignment.bottomCenter,
                transform: Matrix4.identity()
                  ..setEntry(3, 2, 0.0015)
                  ..rotateX(-1.5708 * upperT), // 0 to -90 degrees
                child: _HalfCard(
                  width: width,
                  height: height,
                  value: oldValue,
                  isTop: true,
                ),
              ),
            ),
          // 5. Divider Line
          Align(
            alignment: Alignment.center,
            child: Container(
              height: 2,
              width: width,
              color: const Color(0xFF1B1B1B),
            ),
          ),
        ],
      ),
    );
  }
}

class _HalfCard extends StatelessWidget {
  const _HalfCard({
    required this.width,
    required this.height,
    required this.value,
    required this.isTop,
  });

  final double width;
  final double height;
  final int value;
  final bool isTop;

  @override
  Widget build(BuildContext context) {
    final fontSize = height * 0.7;

    return ClipRect(
      child: Align(
        alignment: isTop ? Alignment.topCenter : Alignment.bottomCenter,
        heightFactor: 0.5,
        child: Container(
          width: width,
          height: height,
          decoration: BoxDecoration(
            color: const Color(0xFF2A2A2A),
            borderRadius: BorderRadius.circular(16),
            border: Border.all(color: const Color(0xFF404040), width: 1.5),
            boxShadow: const [
              BoxShadow(
                color: Color(0x99000000),
                offset: Offset(4, 8),
                blurRadius: 12,
              ),
            ],
          ),
          alignment: Alignment.center,
          child: Text(
            '$value',
            style: TextStyle(
              color: const Color(0xFFE8E8E8),
              fontSize: fontSize,
              fontWeight: FontWeight.w700,
              height: 1.0, // Ensures precise centering
            ),
          ),
        ),
      ),
    );
  }
}
