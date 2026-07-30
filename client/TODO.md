a few things i still need to do
in order of urgency

- integrate glyphon for text rendering (i will NOT be wasting my time doing this - sorry)
- create a websocket system and decode msgs (super easy - 25m max)
- connect socket state to render state (easyish - no et)

as well as some pretty neat features:
- client prediction (instead of waiting for movement response from server assume it succeeds and update local client instantly - roll back if wrong)
- lag compensation (basically generation system - if a client is lagging then adjust based on generation) - definitely prevent this from getting exploited!!
- now that we used wgpu (we get access to WebGPU) we can do some cooler stuff (custom shaders)
