/// Penyimpanan kredensial device (flutter_secure_storage).
///
/// Menyimpan `device_id` + `auth_token` + alamat Host terakhir
/// secara aman (PRD: Local Storage → Secure Storage untuk token).
library;

import 'package:flutter_secure_storage/flutter_secure_storage.dart';

/// Kredensial tersimpan untuk auto-reconnect (PRD FR-6).
class SavedCredentials {
  const SavedCredentials({
    required this.host,
    required this.port,
    required this.deviceId,
    required this.authToken,
  });

  final String host;
  final int port;
  final String deviceId;
  final String authToken;
}

class CredentialStore {
  CredentialStore([FlutterSecureStorage? storage])
      : _storage = storage ?? const FlutterSecureStorage();

  static const _kHost = 'dashkey_host';
  static const _kPort = 'dashkey_port';
  static const _kDeviceId = 'dashkey_device_id';
  static const _kAuthToken = 'dashkey_auth_token';

  final FlutterSecureStorage _storage;

  Future<void> save(SavedCredentials creds) async {
    await _storage.write(key: _kHost, value: creds.host);
    await _storage.write(key: _kPort, value: creds.port.toString());
    await _storage.write(key: _kDeviceId, value: creds.deviceId);
    await _storage.write(key: _kAuthToken, value: creds.authToken);
  }

  Future<SavedCredentials?> read() async {
    final host = await _storage.read(key: _kHost);
    final portStr = await _storage.read(key: _kPort);
    final deviceId = await _storage.read(key: _kDeviceId);
    final authToken = await _storage.read(key: _kAuthToken);

    if (host == null || deviceId == null || authToken == null) return null;
    return SavedCredentials(
      host: host,
      port: int.tryParse(portStr ?? '') ?? 48484,
      deviceId: deviceId,
      authToken: authToken,
    );
  }

  Future<void> clear() async {
    await _storage.delete(key: _kHost);
    await _storage.delete(key: _kPort);
    await _storage.delete(key: _kDeviceId);
    await _storage.delete(key: _kAuthToken);
  }
}
