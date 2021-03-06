const { readContractsFile } = require('../ethereum');
const { Token } = require('./token');

class CashToken extends Token {
  constructor(cashToken, proxyAdmin, cashImpl, proxy, liquidityFactor, owner, ctx) {
    super('cash', 'CASH', 'Cash Token', 6, 'CASH', liquidityFactor, cashToken, owner, ctx);

    this.cashToken = cashToken;
    this.proxyAdmin = proxyAdmin;
    this.cashImpl = cashImpl;
    this.proxy = proxy;
  }

  toTrxArg() {
    return `CASH`;
  }

  toWeiAmount(tokenAmount) {
    if (tokenAmount === 'Max' || tokenAmount === 'MAX') {
      return tokenAmount;
    } else {
      return super.toWeiAmount(tokenAmount)
    }
  }

  lockEventName() {
    return 'LockedCash';
  }

  async cashIndex() {
    return this.cashToken.methods.getCashIndex().call();
  }

  async getCashPrincipal(actorLookup) {
    let actor = this.ctx.actors.get(actorLookup);
    return Number(await this.token.methods.cashPrincipal(actor.ethAddress()).call());
  }

  async getTotalCashPrincipal() {
    return Number(await this.token.methods.totalCashPrincipal().call());
  }

  async getCashYieldAndIndex() {
    // TODO: How to parse result?
    return await this.token.methods.cashYieldAndIndex().call();
  }
}

async function buildCashToken(cashTokenInfo, ctx, owner) {
  ctx.log("Deploying cash token...");

  let proxyAdmin = await ctx.eth.__deploy('ProxyAdmin', [], { from: ctx.eth.root() });
  let cashImpl = await ctx.eth.__deploy('CashToken', [owner]);
  let proxy = await ctx.eth.__deploy('TransparentUpgradeableProxy', [
    cashImpl._address,
    proxyAdmin._address,
    cashImpl.methods.initialize(ctx.__initialYield(), ctx.__initialYieldStart()).encodeABI()
  ], { from: ctx.eth.root() });
  let cashToken = await ctx.eth.__getContractAt('CashToken', proxy._address);

  return new CashToken(cashToken, proxyAdmin, cashImpl, proxy, cashTokenInfo.liquidity_factor, owner, ctx);
}

module.exports = {
  CashToken,
  buildCashToken
};
