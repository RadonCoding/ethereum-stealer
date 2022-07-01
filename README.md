# Ethereum Stealer
Generate random ethereum private keys and then drain funds if any.

### How it works
It generates random private keys / seeds and then takes drains funds from them

### How to use
1. Open it in VS Code or your preferred IDE
2. Goto `constants.rs` and replace the address with your own
3. Make sure to change the `INFURA_PROJECT_ID` to your own
4. Run (x64) `cargo build --release` or (x86) `cargo build --release --target=i686-pc-windows-msvc`

### Contributing
1. Fork it
2. Create your branch (`git checkout -b my-change`)
3. Commit your changes (`git commit -am 'changed something'`)
4. Push to the branch (`git push origin my-change`)
5. Create new pull request

### Disclaimer
You will most likely not get anything from this. This project was made for fun.

Based on: https://tms-dev-blog.com/build-a-crypto-wallet-using-rust/
