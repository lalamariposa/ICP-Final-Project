use candid::{ CandidType, Decode, Deserialize, Encode };
use ic_stable_structures::memory_manager::{ MemoryId, MemoryManager, VirtualMemory };
use ic_stable_structures::{ BoundedStorable, DefaultMemoryImpl, StableBTreeMap, Storable };
use std::{ borrow::Cow, cell::RefCell };
use candid::Principal;

type Memory = VirtualMemory<DefaultMemoryImpl>;

const MAX_VALUE_SIZE: u32 = 5000;

#[derive(CandidType)]
enum BidError {
    ItemIsNotActive,
    InvalidAmount,
    NoSuchItem,
    AccessRejected,
    UpdateError,
}

#[derive(CandidType, Deserialize)]
struct Item {
    description: String,
    currentHighestBid: u32,
    currentHighestBidder: u32,
    is_active: bool,
    bidders: Vec<candid::Principal>,
    owner: candid::Principal,
}

#[derive(CandidType, Deserialize)]
struct CreateItem {
    description: String,
    is_active: bool,
}

impl Storable for Item {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap());
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap();
    }
}

impl BoundedStorable for Item {
    const MAX_SIZE: u32 = MAX_VALUE_SIZE;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    //list of items
    static ITEM_MAP: RefCell<StableBTreeMap<u64, Item, Memory>> = RefCell::new(
        StableBTreeMap::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))))
    );
}

//get an item
#[ic_cdk::query]
fn get_item(key: u64) -> Option<Item> {
    ITEM_MAP.with(|p| p.borrow().get(&key))
}

//get the length of items
#[ic_cdk::query]
fn get_item_count() -> u64 {
    ITEM_MAP.with(|p| p.borrow().len())
}

//get a list of items
#[ic_cdk::query]
fn get_list_of_items() -> Vec<&Item> {
    // for (k, v) in ITEM_MAP.iter() {
    //     ITEM_MAP.with(|p| p.borrow().get(k));
    // }
    let item_map = ITEM_MAP.borrow_mut();
    let items: Vec<&Item> = item_map.values().collect();
    items
}

//get the item sold for the most
fn most_expensive_item<K, V>(item_map: &StableBTreeMap<K, V>) -> Option<&V> where V: Ord {
    item_map
        .iter()
        .max_by(|a, b| a.1.currentHighestBid.cmp(&b.1.currentHighestBid))
        .map(|(_k, v)| v)
}

//get the item that has been bid on the most
fn most_bidded_item<K, V>(item_map: &StableBTreeMap<K, V>) -> Option<&V> where V: Ord {
    item_map
        .iter()
        .max_by(|a, b| a.1.bidders.len().cmp(&b.1.bidders.len()))
        .map(|(_k, v)| v)
}

#[ic_cdk::update]
fn create_item(key: u64, item: CreateItem) -> Option<Item> {
    let value: Item = Item {
        description: item.description,
        currentHighestBid: 0u32,
        currentHighestBidder: "",
        is_active: item.is_active,
        bidders: vec![],
        owner: ic_cdk::caller(),
    };

    ITEM_MAP.with(|p| p.borrow_mut().insert(key, value))
}

#[ic_cdk::update]
fn edit_item(key: u64, item: CreateItem) -> Result<(), BidError> {
    ITEM_MAP.with(|p| {
        //get item from StableBTreeMap
        let old_item_opt = p.borrow().get(&key);
        let old_item = match old_item_opt {
            Some(value) => value,
            None => {
                return Err(BidError::NoSuchItem);
            }
        };

        //check the owner
        if ic_cdk::caller() != old_item.owner {
            return Err(BidError::AccessRejected);
        }

        //create edited item
        let value = Item {
            description: item.description,
            currentHighestBid: old_item.currentHighestBid,
            currentHighestBidder: old_item.currentHighestBidder,
            is_active: item.is_active,
            bidders: old_item.bidders,
            owner: ic_cdk::caller(),
        };

        //insert it back
        let res = p.borrow().insert(key, value);

        match res {
            Some(_) => Ok(()),
            None => Err(BidError::UpdateError),
        }
    })
}

#[ic_cdk::update]
fn end_item(key: u64) -> Result<(), BidError> {
    ITEM_MAP.with(|p| {
        //get item from StableBTreeMap
        let item_opt = p.borrow().get(&key);
        let mut item = match item_opt {
            Some(value) => value,
            None => {
                return Err(BidError::NoSuchItem);
            }
        };

        //check the owner
        if ic_cdk::caller() != item.owner {
            return Err(BidError::AccessRejected);
        }

        //stop listing the item
        item.is_active = false;
        item.owner = item.currentHighestBidder;
        item.currentHighestBid = 0u32;
        item.currentHighestBidder = "";
        item.bidders = vec![];

        //insert it back
        let res = p.borrow().insert(key, item);

        match res {
            Some(_) => Ok(()),
            None => Err(BidError::UpdateError),
        }
    })
}

#[ic_cdk::update]
fn bid(key: u64, bid_amount: u32) -> Result<(), BidError> {
    ITEM_MAP.with(|p| {
        //get item from StableBTreeMap
        let item_opt = p.borrow().get(&key);
        let mut item = match item_opt {
            Some(value) => value,
            None => Err(BidError::NoSuchItem),
        };

        let caller: Principal = ic_cdk::caller();

        //check if item is active
        if item.is_active == false {
            return Err(BidError::ItemIsNotActive);
        }

        //if caller bids higher then the previous bidders edit the item
        if bid_amount > item.currentHighestBid {
            item.currentHighestBid = bid_amount;
            item.currentHighestBidder = caller;
        }

        item.bidders.push(caller);

        let res = p.borrow_mut().insert(key, item);
        match res {
            Some(_) => Ok(()),
            None => Err(BidError::UpdateError),
        }
    })
}
