/**
 * E2E Encryption for WebRTC media frames using RTCRtpScriptTransform.
 *
 * When an SFU relay is used, the SFU sees only encrypted RTP payloads.
 * In mesh mode (no SFU), DTLS-SRTP already provides E2E encryption,
 * but this layer adds defense-in-depth.
 *
 * Uses AES-256-GCM to encrypt/decrypt encoded media frames.
 */

// 4-byte frame counter used as part of the IV to ensure uniqueness
let encryptFrameCounter = 0;

/**
 * Check if RTCRtpScriptTransform is available in this browser/webview.
 */
export function isE2EESupported(): boolean {
  return typeof RTCRtpScriptTransform !== "undefined";
}

/**
 * Derive an AES-256-GCM key from a room key string.
 */
export async function deriveMediaKey(
  roomKey: string
): Promise<CryptoKey> {
  const enc = new TextEncoder();
  const keyMaterial = await crypto.subtle.importKey(
    "raw",
    enc.encode(roomKey),
    "HKDF",
    false,
    ["deriveKey"]
  );

  return crypto.subtle.deriveKey(
    {
      name: "HKDF",
      hash: "SHA-256",
      salt: enc.encode("chatr-e2ee-media-v1"),
      info: enc.encode("media-encryption"),
    },
    keyMaterial,
    { name: "AES-GCM", length: 256 },
    false,
    ["encrypt", "decrypt"]
  );
}

/**
 * Encrypt an encoded video/audio frame.
 * Prepends a 12-byte IV to the ciphertext.
 */
export async function encryptFrame(
  key: CryptoKey,
  frame: RTCEncodedVideoFrame | RTCEncodedAudioFrame
): Promise<void> {
  const data = frame.data;

  // Build 12-byte IV: 8 random bytes + 4-byte counter
  const iv = new Uint8Array(12);
  crypto.getRandomValues(iv.subarray(0, 8));
  const counter = encryptFrameCounter++;
  new DataView(iv.buffer).setUint32(8, counter);

  const encrypted = await crypto.subtle.encrypt(
    { name: "AES-GCM", iv },
    key,
    data
  );

  // Output: [12-byte IV][ciphertext+tag]
  const output = new ArrayBuffer(iv.byteLength + encrypted.byteLength);
  const outputView = new Uint8Array(output);
  outputView.set(iv, 0);
  outputView.set(new Uint8Array(encrypted), iv.byteLength);

  frame.data = output;
}

/**
 * Decrypt an encoded video/audio frame.
 * Reads the prepended 12-byte IV from the ciphertext.
 */
export async function decryptFrame(
  key: CryptoKey,
  frame: RTCEncodedVideoFrame | RTCEncodedAudioFrame
): Promise<void> {
  const data = new Uint8Array(frame.data);

  if (data.byteLength < 12) {
    // Too small to contain IV, pass through
    return;
  }

  const iv = data.slice(0, 12);
  const ciphertext = data.slice(12);

  try {
    const decrypted = await crypto.subtle.decrypt(
      { name: "AES-GCM", iv },
      key,
      ciphertext
    );
    frame.data = decrypted;
  } catch {
    // Decryption failed - frame may be unencrypted or from a peer
    // with a different key. Pass through silently.
  }
}

/**
 * Attach E2E encryption transforms to an RTCRtpSender.
 * Only works when RTCRtpScriptTransform is available.
 */
export function attachSenderTransform(
  sender: RTCRtpSender,
  key: CryptoKey
): void {
  if (!isE2EESupported()) return;

  // Use Encoded Transform API (streams-based approach)
  const senderStreams = (sender as any).createEncodedStreams?.();
  if (!senderStreams) return;

  const { readable, writable } = senderStreams;
  const transformStream = new TransformStream({
    async transform(frame: RTCEncodedVideoFrame | RTCEncodedAudioFrame, controller: TransformStreamDefaultController) {
      await encryptFrame(key, frame);
      controller.enqueue(frame);
    },
  });

  readable.pipeThrough(transformStream).pipeTo(writable);
}

/**
 * Attach E2E decryption transforms to an RTCRtpReceiver.
 * Only works when RTCRtpScriptTransform is available.
 */
export function attachReceiverTransform(
  receiver: RTCRtpReceiver,
  key: CryptoKey
): void {
  if (!isE2EESupported()) return;

  const receiverStreams = (receiver as any).createEncodedStreams?.();
  if (!receiverStreams) return;

  const { readable, writable } = receiverStreams;
  const transformStream = new TransformStream({
    async transform(frame: RTCEncodedVideoFrame | RTCEncodedAudioFrame, controller: TransformStreamDefaultController) {
      await decryptFrame(key, frame);
      controller.enqueue(frame);
    },
  });

  readable.pipeThrough(transformStream).pipeTo(writable);
}
