# Differences with ANCHOR contracts (work in progress)

In this file, you will find all the differences between this repository and the original ANCHOR repository, located at : https://github.com/Anchor-Protocol/anchor-bAsset-contracts/


## Hub
	Major changes : 
		1. Added the validator registry. 
			Those changes were made on lido's repository : https://github.com/lidofinance/lido-terra-contracts
			This HUB contract is rather closer to the https://github.com/lidofinance/lido-terra-contracts/tree/main/contracts/lido_terra_hub (without stLuna included)
		2. Removed airdrop functionality. aLuna stakers won't be able to claim airdrops, they will stay attached to the hub contract
	Lots of things to change in test to account for that new validator registry functionality.
	Removed all airdrops related tests
	What was done here was to add the validator registry functionality from lido into anchor's hub

## Reward
	Major Changes : 
		1. The price querier and native swaps don't exist anymore
			We had to switch to price queries and swaps using Terra dexes. 
			We chose 3 sources of truth for the price of the asset --> Astroport, PhoenixSwap and TerraSwap
	Some changes in tests and syntax. Especially removing the native swap and adding liquidity swaps.
	Removed the tax, as there is no more native assets tax on Terra 2.0

## Token
	Major Changes : 
		None
	Modified variable names, switched to Addr instead of Canonical Addresses.

## Validator registry
	Major Changes : 
		None
	Very few changes were made on this contract compared to : https://github.com/lidofinance/lido-terra-contracts/tree/main/contracts/lido_terra_validators_registry