# MWC-wallet coinjoin pool

In a context of a low number of on chain transactions, Dandelion is not really effective. To improve privacy, MWC-wallet will do a coinjoin at wallet level.  MWC-wallet will talk with other participating mwc-wallets and built a 'multikernel' transaction.

The mwc-wallet coinjoin will take some time to process in order to capture enough transactions from honest wallet participants.

This method can allow to obfuscate regular transactions, but it's not the primary use case for multiple reasons. In case of Non interactive transactions (NIT) it will be easier to use for regular transactions.


## Checking traceability of the outputs.

The mwc-wallet will inspect the blocks and identify the outputs that are potentially traceable.
It will assume outputs are untraceable if the block contain multiple transactions (more then T kernels and T outputs) 
Value of T need to be defined by user, more is better. Value of 5 should give good privacy assuming no observer was tracking transactions from mempool in the past.

Over a certain time period, because of the cut-though, the number of kernels and outputs will decline. Because of that the
wallet will need to track it's old outputs and periodically spend them. The time period will depend on network activity. *** @Suem can confirm

## Coinjoin pool overview

To build the coinjoin transaction, multiple participants should "add up" their transaction details before posting it on-chain, where one participant passes it on to the next until enough participants did this. Therefore none of them will be able to learn information about all their peers. Here is the proposed workflow:

1. Wallet start listening on CoinJoin messages and advertise that it is ready to participate. Part 
   of advertisement should be the number of expected coinjoin transactions (T).
2. Listening to the traffic will allow to learn information about other participants and detect who behave honestly.
3. Over time each participant will be able to build a pool of participants who agree to build a transaction with the same T value.
4. Because T is known, the expected number of participants can be selected as T*2 and any wallet can start the building the transaction.
   The first few participants will pay less fees. For example for T=5 and, of participants 10, assuming all transactions has 1 input, 1 output and 1 kernel, 
   the fees will looks like: `0.000,  0.001,  0.002, 0.003, 0.005, 0.007, 0.008, 0.008, 0.008, 0.008` So the first participant 
   will pay nothing, second will pay 0.001 MWC, the last four will pay 0.008 each. The fees values can vary because participants can include 
   any transactions without limitations. But it is important to understand that first participants are paying much smaller fees then the rest of the pool. 
5. The initiator will build a transaction, for example it can include a self transaction. Select the random participant and send encrypted message to him.
6. Whoever gets the message, will add it's own transaction and aggregate the result. After aggregation it will be impossible to learn 
   how to trace inputs/outputs. Then another participant will be selected and aggregated transaction will be sent.
7. Eventually all 10 participants will be able to add inputs/outputs and result of the aggregation will be posted to the network.
   On Chain this is just another regular coinjoin multikernel transaction like any other.

If some of the participants are dishonest, others will be able to learn that fact by observing the traffic, and are able to remove him 
from the Coinjoin pool and retry. For every attempt to create a coinjoin transaction all participants need to regenerate used outputs.

Please note, there are some natural features of this method:
- Any participant can publish any transaction so even if aggregation fails along the way funds can't get "stuck". But for every publishing attempt all outputs need to be regenerated (if outputs are not regenerated then an attacker can learn the dependency between inputs and outputs).  
- On every step, the aggregated transaction can be verified and published if the participant is willing to add enough fees.
- Second half of participants (in our example 6-10) can publish the transaction for all Members of the group who already participated in the CoinJoin because the value of T is high enough. As a result, if really needed, any of those participants can publish the current Coinjoin transaction with guaranteed result. No potential attacker will be able to interrupt them.

## Messages

Using the libp2p, there will be 3 type of messages:

##### I am online - message 1:
```
{
    "T" : [5,6,10]
    "position" : 0.2
    "pub_key" : "834756342987563429867458654738947356"
}
```
This message contain just a public key of the wallet that want to participate in coinjoin. For privacy and usuability this PubKey can be a one time
temporary key not related to the wallet seed or anything else. Position is telling other peers your desired position in the group. Other participants might respect that,
but still there is no guarantee that you will get this position. 

Implicitly it will have p2p node ID. Only TOR wallet can join P2P network so no data about the wallet is leaked.


##### Instructions - message 2:
```
{
    "nonce" : 376824837256443,
    "recipient" : "rehjtgreioufgdh",  // pub_key  Hash
    "instructions" : "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
}
```
This message has the instructions that is added by somebody and encrypted with Diffie Hellman. Only the Recipient with the specified public key will be able to read the data.

Instructions data is a multikernel transaction that is expected to be updated with additional data.

##### Attack complain - message 3:
```
{
    "message_ID" : "XXXXXXXXXXXXXXXXXXX",
    "nonce" : 376824837256443,
    "recipient" : "rehjtgreioufgdh",  // pub_key  Hash
    "secret" : "XXXXXXXXXXXXXXXXXXX"
    "new_pub_key" : "XXXXXXXXXXX"
}
```
This message can be published to proof that the sender of the faulted message ID was a non honest participant. To proof that, the wallet will need 
to reveal the secret for it's public key. So everybody will be able to read the faulted message and confirm the participant was non honest and remove it from the coinjoin partipants list. 
As a result, the node will need to switch to another public key. Since that public key is not attached to anything, it is possible to do so.

## MWC-Wallet coinjoin workflow.

T - number of expected coinjoined transactions Example: 5

1. The wallet is periodically checking if some of it's outputs are traceable. If at least one is, the wallet goes to the next step.
2. Wallet builds a self spend transaction using the identified traceble outputs.
    - Note, instead of only self spend outputs, the wallet can include any transaction like a regular payment.
3. Wallet joins the libp2p Pub/Sub topic 'CoinJoin' and listens to it.
4. Every 5 minutes the wallet is posting 'I am online message' with it's freshly generated Dalek PubKey to 'CoinJoin' topic.
5. Wallet is listening on 'CoinJoin' and collecting the data
    - if it receive 'I am online - message 1', the list of active wallets PubKey is updated. If list of active wallet is 3x T, then wallet can initiate publishing of it's own transaction.
    - if it getting 'Instructions - message 2', wallet can decode it, validate transactions. If transaction partly published to blockchain by bad actor, **we drop it (mission failed, need to retry)**. Otherwise go to step 7.
6. When wallet collected enough 2x T transactions to publish, including it's own, it will publish all of them. **The mission is successfull** for all participants.
7. If wallet gets 'Instructions - message 2' with it's own transaction, the message will be republished to any 'random' peer (see below how to select honest random peer).
   Because PubKey is known, the message can be encrypted with Diffie Hellman.
8. If wallet gets 'Instructions - message 2' without transaction to mix, the message will be enriched with it's own transaction and republished
   to random peer.
9. Periodically wallet checking if his transactions are published to the blockchain. If it found at Tx Node, **the mission is successfull**

Notes:
> every 'add transaction' requires to do kernel offset. As a result the disaggregation will be impossible.

> the first participant will pay smaller fee the the last one.

## Attacking

Attackers can pursue different goals. Let's check what they can do.

#### Make joining inefficient by publishing too early.

An Attacker can advertise many wallets and every time his wallets are selected, he could publish transactions to the blockchain without including the expected number of participants.
As a result, a weak CoinJoin occurs.

Prevention:
If value of T is consensus, then starting participant can pay smaller fee, the next one will pay more to make the sum expected value.
The node will need to reject smaller fee transactions (will need to check the code). If T is 5, then the for 5x2=10 participants (one input, one output per transaction) fees can be
0.000,  0.001,  0.002, 0.003, 0.005, 0.007, 0.008, 0.008, 0.008, 0.008  (the average fee is 0.005, the sum of all fees have to be 0.05 in order to publish on chain)

As a result, attacker will need at least to add another transaction to pay the remaining mining fees. Honest coinjoin participants 
wallet will later found the undesired outputs traceability value and will automatically retry. But attacker will keep 
paying fees. As a result that will be costly for attacker to do that.

#### Make joining inefficient by dropping everything.

Dropping all request will prevent the Join happen normally.

In this case the wallet can keep tracking of traffic and blacklist p2p nodes that didn't answered. Eventually it will build a black list.
Attacker will need to change the p2p guids. But in this case the wallet will prefer the peers that longer staying online, so attacker node will be out for the session.
<br/>We can proof that eventually all attacker nodes will be detected and only the honest odes will be left.

#### Observing

Attacker can just observe the transaction before merge and try to build input/output mapping.

That will be relatively hard because:
1. An Observer will need to behave as honest node. It should at least republish the traffic because otherwise other wallets will black list it.
2. If there are many observers, then observers will need to participate by including it's own transactions. That will cost fees.
3. Because of 1 and 2, it is possible to have relatively small numbers of observers. As a result instead of T transactions, observer might spot
   smaller number of the merged transactions. But in this case that still ok. Some fraction of outputs can be observed, but
   it is not enough to build the graph who-pay-who. Probability will be very low.

#### Frame other players

Attackers can always send badly encrypted message to the next peer. As a result this player will look like an honest one, but another player will 
look like he's dropping the request. 
As another alternative an attacker can send garbage Data, the result will still be the same. Because traffic is encrypted, that will be impossible to validate.

The mitigation to that is posting an 'Attack complain - message 3'. The node can reveal it's secret, so everybody can learn who the attacker is.
As a result of revealing the secret, the node will need to change it's public key. But because other players will see that, the reputation can be kept. 

# Changes

Here we will track description of the changes at the projects, it will be easier to review teh code. 
