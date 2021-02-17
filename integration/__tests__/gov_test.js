const {
  buildScenarios,
} = require('../util/scenario');
const { decodeCall } = require('../util/substrate');

let gov_scen_info = {
  tokens: [
    { token: "zrx" }
  ],
};

buildScenarios('Gov Scenarios', gov_scen_info, [
  {
    name: "Update Interest Rate Model by Governance",
    scenario: async ({ ctx, zrx, chain, starport }) => {
      let newKink = {
        Kink: {
          zero_rate: 100,
          kink_rate: 500,
          kink_utilization: 80,
          full_rate: 1000
        }
      };
      let extrinsic = ctx.api().tx.cash.updateInterestRateModel(zrx.toChainAsset(), newKink);
      await starport.executeProposal("Update ZRX Interest Rate Model", [extrinsic]);
      expect(await chain.interestRateModel(zrx)).toEqual(newKink);
    }
  },
  {
    name: "Update Auth",
    scenario: async ({ ctx, chain, starport }) => {
      let aliceInitId = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
      let aliceInitEthKey = "0x6a72a2f14577d9cd0167801efdd54a07b40d2b61";
      const newAuths = [[ctx.actors.keyring.decodeAddress(aliceInitId), { eth_address: aliceInitEthKey }]];
      let extrinsic = ctx.api().tx.cash.changeAuthorities(newAuths);
      await starport.executeProposal("Update authorities", [extrinsic]);
      console.log(await chain.pendingCashValidators());
      // await chain.waitUntilSession(3);
      // console.log(await chain.cashValidators());
    }
  },
  {
    name: "Read Extrinsic from Event",
    scenario: async ({ ctx, zrx, chain, starport }) => {
      let newKink = {
        Kink: {
          zero_rate: 100,
          kink_rate: 500,
          kink_utilization: 80,
          full_rate: 1000
        }
      };
      let extrinsic = ctx.api().tx.cash.updateInterestRateModel(zrx.toChainAsset(), newKink);
      let { event } = await starport.executeProposal("Update ZRX Interest Rate Model", [extrinsic]);
      let [[[data]]] = event.data;

      expect(decodeCall(ctx.api(), data)).toEqual({
        section: "cash",
        method: "updateInterestRateModel",
        args: [
          zrx.toChainAsset(true),
          {
            "Kink": {
              "full_rate": "1,000",
              "kink_rate": "500",
              "kink_utilization": "80",
              "zero_rate": "100"
            }
          }
        ]
      });
    }
  }
]);
