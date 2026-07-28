// Copyright 2009-present MongoDB, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0

#include <mongoac/collection.h>

//

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/client.h>
#include <mongoac/database.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>
#include <test_util/bson.hpp>
#include <test_util/owning_ptr.hpp>

#include <array>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("drop", "[mongoac][collection]")
{
   using mongoac::test_util::bson_array_contains_string;

   auto const client =
      make_owning_ptr(mongoac_client_new("mongodb://localhost:27017", nullptr), &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = make_owning_ptr(mongoac_client_get_database(client, "mongoac_collection_drop", nullptr, nullptr),
                                   &mongoac_database_destroy);

   // Clean test state.
   {
      mongoac_database_drop(db, nullptr, nullptr, nullptr);
      {
         auto const names =
            make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);
         REQUIRE(bson_empty0(names.get()));
      }

      mongoac_database_create_collection(db, nullptr, "a", nullptr, nullptr);
      mongoac_database_create_collection(db, nullptr, "b", nullptr, nullptr);
      mongoac_database_create_collection(db, nullptr, "c", nullptr, nullptr);
      {
         auto const names =
            make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);

         CHECK(bson_array_contains_string(names, "a"));
         CHECK(bson_array_contains_string(names, "b"));
         CHECK(bson_array_contains_string(names, "c"));
      }
   }

   auto const coll_a = make_owning_ptr(mongoac_database_get_collection(db, "a", nullptr), &mongoac_collection_destroy);
   auto const coll_b = make_owning_ptr(mongoac_database_get_collection(db, "b", nullptr), &mongoac_collection_destroy);
   auto const coll_c = make_owning_ptr(mongoac_database_get_collection(db, "c", nullptr), &mongoac_collection_destroy);

   SECTION("async")
   {
      auto const drop_b =
         make_owning_ptr(mongoac_collection_drop_async(coll_b, nullptr, nullptr, nullptr), &mongoac_future_destroy);

      mongoac_runtime_block_on(runtime, drop_b, nullptr);
      {
         auto const names =
            make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);

         CHECK(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK(bson_array_contains_string(names, "c"));
      }

      auto const drop_a =
         make_owning_ptr(mongoac_collection_drop_async(coll_a, nullptr, nullptr, nullptr), &mongoac_future_destroy);
      auto const drop_c =
         make_owning_ptr(mongoac_collection_drop_async(coll_c, nullptr, nullptr, nullptr), &mongoac_future_destroy);
      {
         std::array<mongoac_future_t const *, 2u> futures = {drop_a, drop_c};
         mongoac_runtime_block_on_all(runtime, futures.data(), futures.size(), nullptr);
      }
      {
         auto const names =
            make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);

         CHECK_FALSE(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK_FALSE(bson_array_contains_string(names, "c"));
      }
   }

   SECTION("sync")
   {
      mongoac_collection_drop(coll_b, nullptr, nullptr, nullptr);
      {
         auto const names =
            make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);

         CHECK(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK(bson_array_contains_string(names, "c"));
      }

      mongoac_collection_drop(coll_a, nullptr, nullptr, nullptr);
      mongoac_collection_drop(coll_c, nullptr, nullptr, nullptr);
      {
         auto const names =
            make_owning_ptr(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr), &bson_destroy);

         CHECK_FALSE(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK_FALSE(bson_array_contains_string(names, "c"));
      }
   }
}
