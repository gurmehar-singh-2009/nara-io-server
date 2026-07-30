# nara.io

A prototype diep.io clone game.

Features:
- Advanced Anti Cheat
- Add more later as i go

this project was an excuse for me to use the `do yeet` expr.

so i havent actually finished the project yet (not even close) so consider the below information a WIP

## Getting Started
You will need the following installed:
- Rust

And within that you must run these commands:
```bash
cargo install trunk
```

Then you can run
```bash
chmod +x run_client.sh run_server.sh
./run_client.sh
./run_server.sh
```
if you are hosting locally, otherwise use the `prod` variation.

## Crates usage
i will try to refrain from importing crates that are not 1000% necessary (or that i feel like i could enjoyingly replicate functionality of), but when it will save considerable time or i think it's not worth implementing i will import certain crates.

## Note on Cryptography
this project uses a simple x25519 handshake that establishes a chacha20-poly1305 cipher. i could not bring myself to use kyber/ml-kem for this project (however i have used it previously - honestly not worth the hassle).

## Unique features (i think)
- Lua scripting. you can create mini plugins that do stuff. as well as changing configs at runtime.
- Anti cheat. i'm pretty sure there isn't a diep.io clone with behavioural anti cheat, let this serve as the first!

stuff i haven't gotten around to implementing yet (aside from what's mentioned in crate TODO.mds)
- useless client protections. i read if you put stuff in a worker it makes it harder...? still gonna freeze globals and detect [native code] bs but theyre generally pretty easy to bypass (deepseek does it in 1 prompt). supposed to keep skids away if they dont know anything.

## Why use Rust? Why not write in JS/TS, or something else like that?
rust is pretty cool. i like the syntax, crates and overall development cycle.
sure, you prototype faster in typescript but it isn't as rewarding (especially since ai can 1shot typescript code).
not to mention: performance buffs (wasm but counteracted with js interop, server (easy multi-threading)), security buff to client (wasm >>> js for skid protection).
and truly speaking, rust has to be the future for development. better to be familiar with it before wide adoption.

## Contribution Guidelines
i will not be accepting any feature contributions, only contributions that refactor existing code with clear benefits. final decisions will be made by me solely on what code is added, and i reserve the right to use my own discretion regardless.

## Ai usage
ai was used for converting formats like json to toml, and some feature suggestions.
i'm refusing to use it for anything else since i want to learn from this project.
however, i did use ai to debug certain parts, below is a list i will keep updated with files ai has contaminated:
- /client/src/render/shader/fragment.wgsl
