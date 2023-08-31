# ICP Final Project

This final project is a basic Auction smart contract. In this contract, users will be able to:

- List Items
- Bid for an item
- Update the listing of an item
- Stop the listing of an item

Listing and Bidding: Users will be able to list an item. After listing the item, other users can bid for it. Bids will be held in a StableBTreeMap, allowing visibility into which principal bid how much.

Editing and Stopping Listings: The owner of the item can edit the listing or stop the process at any time. When the process is stopped, the highest bidder will become the owner of the item.

Item Management: You will maintain a list of items. Implement the necessary query methods to retrieve a specific item, a list of items, the length of items listed on the contract, the item sold for the most, and the item that has been bid on the most.

Security Checks: Implement basic security checks to ensure that only the owner of the listing can update or stop it.

If you want to start working on this project right away, you might want to try the following commands:

```bash
git clone https://github.com/lalamariposa/ICP-Final-Project.git
cd ICP_Final-Project/
```

## Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start --background

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

Once the job completes, your application will be available at `http://localhost:4943?canisterId={asset_canister_id}`.

If you have made changes to your backend canister, you can generate a new candid interface with

```bash
npm run generate
```

at any time. This is recommended before starting the frontend development server, and will be run automatically any time you run `dfx deploy`.

If you are making frontend changes, you can start a development server with

```bash
npm start
```

Which will start a server at `http://localhost:8080`, proxying API requests to the replica at port 4943.
