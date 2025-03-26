// Truncate Solana wallet address for display
export const truncateAddress = (address: string): string => {
  if (!address) return '';
  return `${address.slice(0, 4)}...${address.slice(-4)}`;
};

// Format lamports to SOL with appropriate decimal places
export const formatSol = (lamports: number): string => {
  const sol = lamports / 1_000_000_000; // 1 SOL = 10^9 lamports
  return sol.toLocaleString(undefined, {
    minimumFractionDigits: 2,
    maximumFractionDigits: 6,
  });
};

// Format token amounts with appropriate decimal places
export const formatTokenAmount = (amount: number): string => {
  return amount.toLocaleString(undefined, {
    minimumFractionDigits: 0,
    maximumFractionDigits: 6,
  });
}; 