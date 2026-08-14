// Copyright 2009-present MongoDB, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0

#include <mongoac/database.h>

//

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/client.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>
#include <test_util/bson.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

#include <array>

using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::owning_bson;

TEST_CASE("create_collection", "[mongoac][database]")
{
   using mongoac::test_util::bson_array_contains_string;
   using mongoac::test_util::to_mongoac;

   auto const client =
      make_owning_ptr(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), nullptr), &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = make_owning_ptr(
      mongoac_client_get_database(client, to_mongoac("mongoac_database_create_collection_async"), nullptr, nullptr),
      &mongoac_database_destroy);

   // Clean test state.
   {
      mongoac_database_drop(db, nullptr, nullptr, nullptr);
      auto const names = owning_bson(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr));
      REQUIRE(bson_empty(names.bson_ptr()));
   }

   SECTION("async")
   {
      auto const a =
         make_owning_ptr(mongoac_database_create_collection_async(db, nullptr, to_mongoac("a"), nullptr, nullptr),
                         &mongoac_future_destroy);
      auto const b =
         make_owning_ptr(mongoac_database_create_collection_async(db, nullptr, to_mongoac("b"), nullptr, nullptr),
                         &mongoac_future_destroy);
      auto const c =
         make_owning_ptr(mongoac_database_create_collection_async(db, nullptr, to_mongoac("c"), nullptr, nullptr),
                         &mongoac_future_destroy);

      {
         std::array<mongoac_future_t const *, 3u> futures = {a, b, c};
         mongoac_runtime_block_on_all(runtime, futures.data(), futures.size(), nullptr);
      }

      auto const names = owning_bson(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr));

      CHECK(bson_array_contains_string(names, "a"));
      CHECK(bson_array_contains_string(names, "b"));
      CHECK(bson_array_contains_string(names, "c"));
   }

   SECTION("sync")
   {
      mongoac_database_create_collection(db, nullptr, to_mongoac("a"), nullptr, nullptr);
      mongoac_database_create_collection(db, nullptr, to_mongoac("b"), nullptr, nullptr);
      mongoac_database_create_collection(db, nullptr, to_mongoac("c"), nullptr, nullptr);

      auto const names = owning_bson(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr));

      CHECK(bson_array_contains_string(names, "a"));
      CHECK(bson_array_contains_string(names, "b"));
      CHECK(bson_array_contains_string(names, "c"));
   }
}
