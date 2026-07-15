export const MAX_FRAME_BYTES = 64 * 1024;

export function encodeFrame(value) {
  const body = Buffer.from(JSON.stringify(value), "utf8");
  if (body.length > MAX_FRAME_BYTES) throw new Error(`frame too large: ${body.length}`);
  const frame = Buffer.allocUnsafe(4 + body.length);
  frame.writeUInt32BE(body.length, 0);
  body.copy(frame, 4);
  return frame;
}

export function createFrameDecoder(onFrame) {
  let buffered = Buffer.alloc(0);
  return (chunk) => {
    buffered = Buffer.concat([buffered, Buffer.from(chunk)]);
    while (buffered.length >= 4) {
      const length = buffered.readUInt32BE(0);
      if (length > MAX_FRAME_BYTES) throw new Error(`frame too large: ${length}`);
      if (buffered.length < 4 + length) return;
      const body = buffered.subarray(4, 4 + length);
      buffered = buffered.subarray(4 + length);
      onFrame(JSON.parse(body.toString("utf8")));
    }
  };
}

