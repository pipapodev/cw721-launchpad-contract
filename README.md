# CW721 Launchpad Contract

A launchpad manager contract to sell your cw721 base contract. Set this contract as the minter.

## Metadata preparation for a mint

```rust
struct Launch {
    owner_address: Addr,
    contract_address: Addr,
    max_supply: u64,
    base_uri: String,
    is_base_uri_static: bool,
    media_extension: Option<String>,
    whitelist_price: Uint128,
    whitelist_max_buy: u16,
    whitelist_started_at: u64,
    whitelist_ended_at: u64,
    public_price: Uint128
    public_max_buy: u16,
    public_started_at: u64,
    public_ended_at: u64,
    price_denom: String
}
```
