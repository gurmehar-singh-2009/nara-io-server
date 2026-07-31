a few things i still need to do
in order of urgency

[-] integrate glyphon for text rendering (i will NOT be wasting my time doing this - sorry)
    note for this: glyphon sucks bro, there is no built in text borders so either i have to roll my own with sdf or do like 10 calls to support borders by shifting the test a little and making it the border color thenr endering the text ontop of it. had to redesign a little bit as well to support text vs non text render entities.
- create a websocket system and decode msgs (super easy - 25m max)
- connect socket state to render state (easyish - no et)

as well as some pretty neat features:
- client prediction (instead of waiting for movement response from server assume it succeeds and update local client instantly - roll back if wrong)
- lag compensation (basically generation system - if a client is lagging then adjust based on generation) - definitely prevent this from getting exploited!!
- now that we used wgpu (we get access to WebGPU) we can do some cooler stuff (custom shaders)

also, for a little more security use either:
https://github.com/open-obfuscator/o-mvll
https://github.com/obfuscator-llvm/obfuscator/tree/llvm-4.0