import assert from "node:assert/strict";
import { test } from "node:test";
import { groupOffers, dealUrl } from "../src/lib/store-search.ts";

test("sorts valid offers first and retains stores with no coverage", () => {
  const stores = [
    { storeID: "1", storeName: "Steam", isActive: 1 },
    { storeID: "2", storeName: "GOG", isActive: 1 },
    { storeID: "3", storeName: "Fanatical", isActive: 1 },
    { storeID: "4", storeName: "Closed", isActive: 0 },
  ];
  const deal = (storeID: string, price: string) => ({ storeID, price, dealID: "abc", retailPrice: "20" });
  const rows = groupOffers(stores, [deal("1", "20"), deal("2", "invalid"), deal("3", "10"), deal("3", "0")]);
  assert.deepEqual(rows.map((row) => row.storeName), ["Fanatical", "Steam", "GOG"]);
  assert.equal(rows[0].deals[0].price, "0");
  assert.equal(rows[2].deals.length, 0);
});

test("purchase IDs are encoded once, preserving the required provider redirect", () => {
  assert.equal(dealUrl("a%2Bb%3D"), "https://www.cheapshark.com/redirect?dealID=a%2Bb%3D");
  assert.equal(dealUrl("a+b="), dealUrl("a%2Bb%3D"));
});
