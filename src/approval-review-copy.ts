import type { OrderSide, OrderType } from '../shared/ipc-types.ts';

export function approvalReviewMoneyCopy(side: OrderSide, orderType: OrderType) {
  if (side === 'SELL' && orderType === 'LIMIT') {
    return {
      expectedLabel: 'Gross proceeds at limit price (scenario)',
      disclosure: 'A sell limit sets the minimum sale price. This scenario is not a maximum on proceeds or fees.',
    } as const;
  }

  if (side === 'SELL' && orderType === 'MARKET') {
    return {
      expectedLabel: 'Estimated gross proceeds',
      maximumLabel: 'Maximum authorized trade value',
      disclosure: 'TradeX checks this gross-value ceiling before submission. The broker does not enforce it as a market-order bound, and it does not cap final proceeds or fees.',
    } as const;
  }

  return {
    expectedLabel: 'Expected spend',
    maximumLabel: 'Maximum authorized spend',
  } as const;
}
