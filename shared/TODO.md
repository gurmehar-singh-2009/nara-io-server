okay so i need to keep physics in this shared crate so its easier to do client prediction.

here are the things i wrote for security:
- Randomize packet IDs based on a seed based on the time.
- Add random junk (packet chaffing) to randomize byte offsets
