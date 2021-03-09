# Atomic Swap Marketplace

The Atomic Swap Marketplace will use a publisher/subscriber model for message pool maintenance. Mwc-wallets that are participating in the marketpace need to listen for new messages.

The primary concern is how to mitigate the non honest players that can come to the market spaming offers they don't plan to execute.

Here are features that should minimize this problem:

1. Integrity fee. In order to have the right to use the marketplace and publish offers, the trader will need to pay an integrity fee to the miners. This integrity fee will need to be paid daily. The
   integrity fee amount will depend on the offer amount and can't be smaller then a current transaction fee 0.01 MWC.
   There are will be three fee levels, for example:
    - Low: 1 bps or 0.01% of the trade amount.
    - Normal: 10 bps or 0.1% of the trade amount.
    - High: 50 bps or 0.5% of the trade amount.

2. Create a local lock transaction when the offer is published. When the atomic swap offer is published, the wallet will create the transaction that locks the
   needed outputs in the wallet. As a result, the user will not be able to create offers if the required funds are not available.
   <br>
   > Please Note that the funds will be locked locally on the wallet level, not on blockchain level.

3. Atomic Swap trade will be only cancellable until locking step. As long as an offer is accepted and initial
   exchange message is done, there will be no way to cancel the trade from UI. Users can still disconnect from the network or kill the wallet process but in this case they will not be able to use the wallet until the atomic swap trade times out and expires.

4. The counterparty that accepts the offer will have to lock their funds first.

5. Black list. Every wallet will be able to maintain a blacklist of bad traders locally. It is possible to recognize them by
   'integrity fee' and by trust score swap deals.



### Placing/getting the offers.

When qt-wallet is publishing an offer, it will send the message every few minutes.
```
{
  "integrity_kernel": {
      "excess" : "6748369356438965854643856784356",
      "signature" : "47564387643857634875683476538765"
  }
    
  "peer_address" : "dkjsdskjh dsfakjhdfskljh", 
  "time" : 328768536,      // current time  
  "currency" : "BTC",
  ...  swap trade deal detals,
  
  ]
}
```
Every message is unencrypted and contains the information about the atomic swap offer.

'integrity_kernel' is a regular transaction kernel with special fee, it must be published with each atomic swap offer. Every receiver wallet can
verify the ownership and fees for this integrity_kernel.
Such integrity fee allows wallets to set filtering rules, so that potential spam offers, with low fees, can be filtered out.
The purpose of this output is to prevent massive flooding. In order to flood the network, an attacker will need to pay fees for each offer. Since fees are paid to miners, in case of spam attacks to the marketplace, miners will start getting more rewards, more miners will join the network, and it will become stronger.

It will take a few minutes for Qt-wallet to open on market place and to receive all offers that exist.
The Proposed value for this time period is 5 minutes.

In order to start listening, the wallet needs to obtain list of kernels from the node that was published during the last 24 hours.
Any of those kernels with fee higher then minimal, can be used as Integrity kernel.

Every offer can be quickly validated by checking integrity_kernel => public_key.
The message can be calculated as 'peer_address' + 'time' from the offer message. The
integrity_kernel => signature must match that message and public key.

