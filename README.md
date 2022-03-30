# NFT Lending

While DAO are formed based on NFT collections, there are far less members than the total amount of NFTs. This is because members tend to hold more than one.

Members go long by holding several NFTs.

The concept of NFT lending is that someone can loan a NFT, while being safe for both parties. The lender is protected by the collateral and the borrower is protected by the contract, which holds his collateral until the default at timestamp is reached.

Two parties can make a loan agreement and execute it on-chain.

Obviously to account for the illiquid character of NFTs and sudden price move, the collateral should be generous in comparison to the current value of the NFT.

Disclaimer: this builds but is totally untested

## Contract Methods

Loan

Creation of the loan agreement, the lender decides the collateral amount and its mint, as well as the default at timestamp

Borrow

Borrower accepts the loan, deposits the collateral and gets the loan transfered to his token account

Repay

Borrower returns the loan, he gets back his collateral in exchange
Repayment is possible until liquidation, so that possibly a late repayment can be made, as an agreement

Close

If nobody takes the loan it can be closed, or the loan can be closed after repayment or after liquidation

If the borrower fails to repay by default at, the lender can liquidate the collateral by calling this method to end the loan agreement

# What is next?

- Add a fee on the loan agreement, so that there is an incentive to loan NFT to people. To make the DAO more open and liquid.
- Add another program which acts as a vault in which multiple tokens can be deposited and mint a key token, so that 1 or several tokens can be lent for one or several tokens, NFTs and/or fungible.
- Cleanup seeds so that the code is DRY