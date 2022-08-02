# Function List

* `initialize( usdc address )` - To be Executed once in entire lifetime. Init's a new account which stores usdc address and address of the user which calls this function

* `initialize_project( project ID, project details)` - To be executed by a project owner - creates a new project. Takes Project details + a unique project ID. If project ID is repeated from the past then the function call should fail. Also creates a USDC project vault which stores the money earned while minting CBD, Define the total number of types of CBD(max 20)

* `whitelist( project ID, address to be whitelisted)` - To be executed by project owner to whitelist an address- creates a PDA for the whitelisted address which stores the number CBD's of project minted by the address. In this function it is initialized to 0

* `initialize_cbd( project ID, Type)` - Creates a PDA of the type - needed to be done to create PDA with data which all the NFTs of this CBD will read while redemption. When a new NFT of this type is minted - this intruction should be attached to it just before it.

* `mint_cbd( project ID, Type, Vault_bump(bump of project usdc vault))` - Mints CBD after it is initialized. Mints it for the user who called the function and takes the USDC and put it in project vault. Function fails if whitelist PDA account for the user for this project does not exist. If it exists it increases whitelist count by 1. 

* `initialize_redemption( project ID, takes pool addresses, project token )` - Creates a PDA for the project which contains project wide details for the oracle and project token

* `fund_vault( project ID, Type, Amount )` - Fund the specific type with project token

* `redeem_cbd( project ID, Type, Vault_bump(bump of project token vault for it's type), NFT address)` - Burns the NFT, Givens the project token to the caller depending on the promised return and checks the time using the time in PDA created druing initialize_CBD

* `withdraw_fund( project ID, vault_bump, _amount)` - Project owner can use it to withdraw USDC stored in the project vault

* `edit_release_time( project ID, Type )` - Project owner can use it to change the release Date

# Test List 

1. Make sure initialize is called before calling anything
2. Initialize project with a new ID
3. Try Initializing the project with the same ID - should fail
4. Initialize a project with a different ID - should pass
5. Whitelist an address
6. Initialize CBD for a specific type
7. Try minting CBD from non whitelisted address - should fail
8. Mint CBD from whitelisted address - should pass
9. Try minting CBD of a type which is not initialized - shoud fail
10. Try funding vault without creation of redemption account - should fail
11. Initialize redeption account for a project
12. Fund a type of CBD for this project
13. Try redeeming CBD of type 1 but pass type 2 in argument - should fail
14. Redeem CBD - check for balances
15. Try Withdraw USDC funds not as project owner - should fail
16. Withdraw USDC funds as project owner
17. Edit release time for specific type of CBD and try withdrawing it - should fail
