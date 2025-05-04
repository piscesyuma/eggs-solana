pub const VAULT_SEED: &'static [u8] = b"vault";

pub const SECONDS_IN_A_DAY: i64 = 60 * 60 * 24;
pub const LAMPORTS_PER_ECLIPSE: u64 = 1_000_000_000;
const TOKEN_DEICMALS_HELPER: u64 = 1_000_000_000; // 9 decimals
pub const MIN_INITIALIZE_TOKEN_AMOUNT: u64 = 1 * LAMPORTS_PER_ECLIPSE;
pub const INITIAL_BURN_TOKEN_AMOUNT: u64 = 10_000 * TOKEN_DEICMALS_HELPER;

pub const MIN: u64 = 1000;
pub const MIN_INITIALIZE_RATIO: u64 = 1;
pub const FEE_BASE_1000: u64 = 1000;
pub const FEES_BUY: u64 = 100;
pub const FEES_BUY_REFERRAL: u64 = 25;
pub const FEES_SELL: u64 = 125;
pub const MAX_SUPPLY: u64 = 50_000_000 * TOKEN_DEICMALS_HELPER; // 5000 0000 tokens