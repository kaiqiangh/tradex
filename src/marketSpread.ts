export type ExactSpreadResult =
  | { status: 'AVAILABLE'; value: string }
  | { status: 'UNAVAILABLE'; reason: string };

type DecimalValue = { coefficient: bigint; scale: number };

function parsePositiveDecimal(value: string): DecimalValue | undefined {
  if (value.length > 64 || !/^(?:0|[1-9]\d*)(?:\.\d+)?$/.test(value)) return undefined;
  const [whole, fraction = ''] = value.split('.');
  const coefficient = BigInt(`${whole}${fraction}`);
  if (coefficient <= 0n) return undefined;
  return { coefficient, scale: fraction.length };
}

function formatDecimal(coefficient: bigint, scale: number): string {
  if (scale === 0) return coefficient.toString();
  const digits = coefficient.toString().padStart(scale + 1, '0');
  const whole = digits.slice(0, -scale);
  const fraction = digits.slice(-scale);
  return `${whole}.${fraction}`;
}

/** Computes ask - bid without converting provider decimal strings to binary floats. */
export function calculateExactSpread(bid?: string | null, ask?: string | null): ExactSpreadResult {
  if (bid == null || ask == null) {
    return { status: 'UNAVAILABLE', reason: 'Unavailable — bid and ask are required' };
  }
  const parsedBid = parsePositiveDecimal(bid);
  const parsedAsk = parsePositiveDecimal(ask);
  if (!parsedBid || !parsedAsk) {
    return { status: 'UNAVAILABLE', reason: 'Unavailable — quote prices are not valid positive decimals' };
  }
  const scale = Math.max(parsedBid.scale, parsedAsk.scale);
  const scaledBid = parsedBid.coefficient * (10n ** BigInt(scale - parsedBid.scale));
  const scaledAsk = parsedAsk.coefficient * (10n ** BigInt(scale - parsedAsk.scale));
  if (scaledAsk < scaledBid) {
    return { status: 'UNAVAILABLE', reason: 'Unavailable — bid exceeds ask' };
  }
  return { status: 'AVAILABLE', value: formatDecimal(scaledAsk - scaledBid, scale) };
}
