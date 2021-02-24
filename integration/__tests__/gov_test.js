const {
  buildScenarios
} = require('../util/scenario');
const { decodeCall } = require('../util/substrate');
const Web3EthAccounts = require('web3');
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
      let extrinsic = ctx.api().tx.cash.setRateModel(zrx.toChainAsset(), newKink);
      await starport.executeProposal("Update ZRX Interest Rate Model", [extrinsic]);
      expect(await chain.interestRateModel(zrx)).toEqual(newKink);
    }
  },
  {
    // only: true,
    name: "Update Auth",
    scenario: async ({ ctx, chain, starport, validators }) => {
      const alice = validators.validatorInfoMap.alice;
      const alice_account_id = alice.aura_key;
      const newAuthsRaw = [{ substrate_id: ctx.actors.keyring.decodeAddress(alice_account_id), eth_address: alice.eth_account }];
      let extrinsic = ctx.api().tx.cash.changeValidators(newAuthsRaw);
      await starport.executeProposal("Update authorities", [extrinsic]);
      const pending = await chain.pendingCashValidators();

      const newAuths = [[alice_account_id, { eth_address: alice.eth_account }]];
      expect(pending).toEqual(newAuths);

      await chain.waitUntilSession(3);
      const newVals = await chain.cashValidators();
      expect(newVals).toEqual(newAuths);

      const newSessionAuths = await chain.sessionValidators();
      expect(newSessionAuths).toEqual([alice_account_id]);

      const grandpaAuths = await chain.getGrandpaAuthorities();
      expect(grandpaAuths).toEqual([alice.grandpa_key]);

      const auraAuths = await chain.getAuraAuthorites();
      expect(auraAuths).toEqual([alice.aura_key]);
    }
  },
  {
    only: true,
    name: "Add Auth",
    scenario: async ({ ctx, chain, starport, validators }) => {
      // spins up new validator charlie and adds to auth set 
      const keyring = ctx.actors.keyring;
      const peer_id = "12D3KooW9qtwBHeQryg9mXBVMkz4YivUsj62g1tYBACUukKToKof";
      const node_key = "0x0000000000000000000000000000000000000000000000000000000000000002";
      const eth_private_key = "0xb1b07e7078273a09c64ef9bd52f49636535ba26624c7c75a57e1286b13c8f7ea";
      const eth_account = "0x9c00B0af5586aE099649137ca6d00a641aD30736";

      const newValidator = await validators.addValidator("Charlie", { peer_id, node_key, eth_private_key, eth_account });

      const newValidatorKeys = await chain.rotateKeys(newValidator);

      const charlie = keyring.createFromUri("//Charlie");
      const charlieCompoundId = charlie.address;
      await chain.setKeys(charlie, newValidatorKeys);

      const {  alice, bob } = validators.validatorInfoMap;
      const toValKeys = (substrateId, ethAccount) => {return  {"substrate_id": keyring.decodeAddress(substrateId), "eth_address": ethAccount} };
      const allAuthsRaw = [
        toValKeys(alice.aura_key, alice.eth_account),
        toValKeys(bob.aura_key, bob.eth_account),
        toValKeys(charlieCompoundId, eth_account),
      ];
      const extrinsic = ctx.api().tx.cash.changeValidators(allAuthsRaw);
      await starport.executeProposal("Update authorities", [extrinsic]);
      // const timer = ms => new Promise(res => setTimeout(res, ms));
      // await timer(30000);
      await chain.waitUntilSession(3);

      const newSessionAuths = await chain.sessionValidators();
      expect(newSessionAuths.sort()).toEqual([alice.aura_key, bob.aura_key, charlieCompoundId].sort());

      const auraAuths = await chain.getAuraAuthorites();
      expect(auraAuths.sort()).toEqual([alice.aura_key, bob.aura_key, newValidatorKeys.aura].sort());

      const grandpaAuths = await chain.getGrandpaAuthorities();
      expect(grandpaAuths.sort()).toEqual([alice.grandpa_key, bob.grandpa_key, newValidatorKeys.grandpa].sort());
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
      let extrinsic = ctx.api().tx.cash.setRateModel(zrx.toChainAsset(), newKink);
      let { event } = await starport.executeProposal("Update ZRX Interest Rate Model", [extrinsic]);
      let [[[data]]] = event.data;

      expect(decodeCall(ctx.api(), data)).toEqual({
        section: "cash",
        method: "setRateModel",
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
