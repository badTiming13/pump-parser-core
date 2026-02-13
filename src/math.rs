pub fn mid_price_sol(virtual_sol_reserves: f64, virtual_token_reserves: f64) -> f64 {
    (virtual_sol_reserves / virtual_token_reserves) * 1e-3
}

pub fn trade_price_sol(sol_amount: f64, token_amount: f64) -> f64 {
    (sol_amount / token_amount) * 1e-3
}

pub fn market_cap_sol(price: f64) -> f64 {
    price * 1_000_000_000f64
}
