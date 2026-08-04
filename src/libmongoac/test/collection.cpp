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
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/client.h>
#include <mongoac/cursor.h>
#include <mongoac/database.h>
#include <mongoac/error.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>
#include <test_util/bson.hpp>
#include <test_util/owning_ptr.hpp>

#include <array>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_owning_ptr;

TEST_CASE("drop", "[mongoac][collection]")
{
   using mongoac::test_util::bson_array_contains_string;

   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_drop", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   {
      mongoac_database_drop(db, nullptr, nullptr, error);
      {
         auto const names =
            REQUIRE_MAKE_OWNING_PTR(mongoac_database_list_collection_names(db, nullptr, nullptr, error), &bson_destroy);
         REQUIRE(bson_empty0(names.get()));
      }

      mongoac_database_create_collection(db, nullptr, "a", nullptr, error);
      mongoac_database_create_collection(db, nullptr, "b", nullptr, error);
      mongoac_database_create_collection(db, nullptr, "c", nullptr, error);
      {
         auto const names =
            REQUIRE_MAKE_OWNING_PTR(mongoac_database_list_collection_names(db, nullptr, nullptr, error), &bson_destroy);

         CHECK(bson_array_contains_string(names, "a"));
         CHECK(bson_array_contains_string(names, "b"));
         CHECK(bson_array_contains_string(names, "c"));
      }
   }

   auto const coll_a =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "a", error), &mongoac_collection_destroy);
   auto const coll_b =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "b", error), &mongoac_collection_destroy);
   auto const coll_c =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "c", error), &mongoac_collection_destroy);

   SECTION("async")
   {
      auto const drop_b = REQUIRE_MAKE_OWNING_PTR(mongoac_collection_drop_async(coll_b, nullptr, nullptr, error),
                                                  &mongoac_future_destroy);

      mongoac_runtime_block_on(runtime, drop_b, error);
      {
         auto const names =
            REQUIRE_MAKE_OWNING_PTR(mongoac_database_list_collection_names(db, nullptr, nullptr, error), &bson_destroy);

         CHECK(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK(bson_array_contains_string(names, "c"));
      }

      auto const drop_a = REQUIRE_MAKE_OWNING_PTR(mongoac_collection_drop_async(coll_a, nullptr, nullptr, error),
                                                  &mongoac_future_destroy);
      auto const drop_c = REQUIRE_MAKE_OWNING_PTR(mongoac_collection_drop_async(coll_c, nullptr, nullptr, error),
                                                  &mongoac_future_destroy);
      {
         std::array<mongoac_future_t const *, 2u> futures = {drop_a, drop_c};
         mongoac_runtime_block_on_all(runtime, futures.data(), futures.size(), error);
      }
      {
         auto const names =
            REQUIRE_MAKE_OWNING_PTR(mongoac_database_list_collection_names(db, nullptr, nullptr, error), &bson_destroy);

         CHECK_FALSE(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK_FALSE(bson_array_contains_string(names, "c"));
      }
   }

   SECTION("sync")
   {
      mongoac_collection_drop(coll_b, nullptr, nullptr, error);
      {
         auto const names =
            REQUIRE_MAKE_OWNING_PTR(mongoac_database_list_collection_names(db, nullptr, nullptr, error), &bson_destroy);

         CHECK(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK(bson_array_contains_string(names, "c"));
      }

      mongoac_collection_drop(coll_a, nullptr, nullptr, error);
      mongoac_collection_drop(coll_c, nullptr, nullptr, error);
      {
         auto const names =
            REQUIRE_MAKE_OWNING_PTR(mongoac_database_list_collection_names(db, nullptr, nullptr, error), &bson_destroy);

         CHECK_FALSE(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK_FALSE(bson_array_contains_string(names, "c"));
      }
   }
}

#define MONGOAC_ERROR_REQUIRE(error)                                 \
   if (1) {                                                          \
      CHECKED_IF(mongoac_error_code(error) != MONGOAC_ERROR_CODE_OK) \
      {                                                              \
         FAIL(mongoac_error_message(error));                         \
      }                                                              \
   } else                                                            \
      ((void)0)

TEST_CASE("insert_one", "[mongoac][collection]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   REQUIRE(runtime);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_insert_one", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   MONGOAC_ERROR_REQUIRE(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const doc = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 1})"), &bson_destroy);

   SECTION("async")
   {
      auto const future = REQUIRE_MAKE_OWNING_PTR(
         mongoac_collection_insert_one_async(coll, nullptr, doc, nullptr, error), &mongoac_future_destroy);

      mongoac_runtime_block_on(runtime, future, error);
      MONGOAC_ERROR_REQUIRE(error);

      auto const result = REQUIRE_MAKE_OWNING_PTR(mongoac_future_get_bson(future, error), &bson_destroy);

      CHECK(result);

      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, result, "insertedId"));
      CHECK(bson_iter_type(&iter) == BSON_TYPE_OID);

      char str[25];
      bson_oid_to_string(bson_iter_oid(&iter), str);
      CHECK_THAT(str, Catch::Matchers::Matches("[0-9a-f]{24}"));
   }

   SECTION("sync")
   {
      auto const result =
         REQUIRE_MAKE_OWNING_PTR(mongoac_collection_insert_one(coll, nullptr, doc, nullptr, error), &bson_destroy);

      CHECK(result);

      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, result, "insertedId"));
      CHECK(bson_iter_type(&iter) == BSON_TYPE_OID);

      char str[25];
      bson_oid_to_string(bson_iter_oid(&iter), str);
      CHECK_THAT(str, Catch::Matchers::Matches("[0-9a-f]{24}"));
   }
}

TEST_CASE("find", "[mongoac][collection]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   MONGOAC_ERROR_REQUIRE(error);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_find", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   MONGOAC_ERROR_REQUIRE(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const filter = make_owning_ptr(bson_from_json(R"({"$or": [{"x": 1}, {"z": 3}]})"), &bson_destroy);
   REQUIRE(filter);

   SECTION("none")
   {
      SECTION("async")
      {
         auto const future = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_find_async(coll, nullptr, filter, nullptr, error), &mongoac_future_destroy);
         MONGOAC_ERROR_REQUIRE(error);

         mongoac_runtime_block_on(runtime, future, error);
         MONGOAC_ERROR_REQUIRE(error);

         auto const cursor = REQUIRE_MAKE_OWNING_PTR(mongoac_future_get_cursor(future, error), &mongoac_cursor_destroy);
         MONGOAC_ERROR_REQUIRE(error);

         CHECK_FALSE(mongoac_cursor_next(cursor, error));
         MONGOAC_ERROR_REQUIRE(error);
      }

      SECTION("sync")
      {
         auto const cursor = REQUIRE_MAKE_OWNING_PTR(mongoac_collection_find(coll, nullptr, filter, nullptr, error),
                                                     &mongoac_cursor_destroy);
         MONGOAC_ERROR_REQUIRE(error);

         CHECK_FALSE(mongoac_cursor_next(cursor, error));
         MONGOAC_ERROR_REQUIRE(error);
      }
   }

   SECTION("some")
   {
      {
         auto const x = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 1})"), &bson_destroy);
         auto const y = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"y": 2})"), &bson_destroy);
         auto const z = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"z": 3})"), &bson_destroy);
         auto docs = std::array<const bson_t *, 3>{{x, y, z}};

         CHECK(REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_insert_many(coll, nullptr, docs.data(), docs.size(), nullptr, error), &bson_destroy));
         MONGOAC_ERROR_REQUIRE(error);
      }

      // Sorting by _id ascending returns the two matches in insertion order;
      // since x is inserted before z, the {x: 1} document precedes {z: 3}.
      auto const options = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"sort": {"_id": 1}})"), &bson_destroy);

      auto const check_two_results = [&](mongoac_cursor_t const *cursor) {
         REQUIRE(mongoac_cursor_next(cursor, error));
         MONGOAC_ERROR_REQUIRE(error);
         auto const first = REQUIRE_MAKE_OWNING_PTR(mongoac_cursor_current(cursor, error), &bson_destroy);
         MONGOAC_ERROR_REQUIRE(error);

         REQUIRE(mongoac_cursor_next(cursor, error));
         MONGOAC_ERROR_REQUIRE(error);
         auto const second = REQUIRE_MAKE_OWNING_PTR(mongoac_cursor_current(cursor, error), &bson_destroy);
         MONGOAC_ERROR_REQUIRE(error);

         CHECK_FALSE(mongoac_cursor_next(cursor, error));
         MONGOAC_ERROR_REQUIRE(error);

         bson_iter_t iter = {};
         REQUIRE(bson_iter_init_find(&iter, first.get(), "x"));
         REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT32);
         CHECK(bson_iter_int32(&iter) == 1);

         REQUIRE(bson_iter_init_find(&iter, second.get(), "z"));
         REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT32);
         CHECK(bson_iter_int32(&iter) == 3);
      };

      SECTION("async")
      {
         auto const future = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_find_async(coll, nullptr, filter, options, error), &mongoac_future_destroy);
         MONGOAC_ERROR_REQUIRE(error);

         mongoac_runtime_block_on(runtime, future, error);
         MONGOAC_ERROR_REQUIRE(error);

         auto const cursor = REQUIRE_MAKE_OWNING_PTR(mongoac_future_get_cursor(future, error), &mongoac_cursor_destroy);
         MONGOAC_ERROR_REQUIRE(error);

         check_two_results(cursor.get());
      }

      SECTION("sync")
      {
         auto const cursor = REQUIRE_MAKE_OWNING_PTR(mongoac_collection_find(coll, nullptr, filter, options, error),
                                                     &mongoac_cursor_destroy);
         MONGOAC_ERROR_REQUIRE(error);

         check_two_results(cursor.get());
      }
   }
}

TEST_CASE("insert_many", "[mongoac][collection]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);
   MONGOAC_ERROR_REQUIRE(error);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_insert_many", nullptr, error), &mongoac_database_destroy);
   MONGOAC_ERROR_REQUIRE(error);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   MONGOAC_ERROR_REQUIRE(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);
   MONGOAC_ERROR_REQUIRE(error);

   auto const x = REQUIRE_MAKE_OWNING_PTR(BCON_NEW("x", BCON_INT32(1)), &bson_destroy);
   auto const y = REQUIRE_MAKE_OWNING_PTR(BCON_NEW("y", BCON_INT32(2)), &bson_destroy);
   auto const z = REQUIRE_MAKE_OWNING_PTR(BCON_NEW("z", BCON_INT32(3)), &bson_destroy);

   auto docs = std::array<const bson_t *, 3>{{x, y, z}};

   auto const count_inserted_ids = [](bson_t const *result) -> int {
      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, result, "insertedIds"));
      REQUIRE(bson_iter_type(&iter) == BSON_TYPE_DOCUMENT);
      bson_iter_t sub = {};
      REQUIRE(bson_iter_recurse(&iter, &sub));
      int n = 0;
      while (bson_iter_next(&sub)) {
         ++n;
      }
      return n;
   };

   SECTION("async")
   {
      auto const future = REQUIRE_MAKE_OWNING_PTR(
         mongoac_collection_insert_many_async(coll, nullptr, docs.data(), docs.size(), nullptr, error),
         &mongoac_future_destroy);
      MONGOAC_ERROR_REQUIRE(error);

      mongoac_runtime_block_on(runtime, future, error);
      MONGOAC_ERROR_REQUIRE(error);

      auto const result = REQUIRE_MAKE_OWNING_PTR(mongoac_future_get_bson(future, error), &bson_destroy);
      MONGOAC_ERROR_REQUIRE(error);

      CHECK(result);
      CHECK(count_inserted_ids(result) == 3);
   }

   SECTION("sync")
   {
      auto const result = REQUIRE_MAKE_OWNING_PTR(
         mongoac_collection_insert_many(coll, nullptr, docs.data(), docs.size(), nullptr, error), &bson_destroy);
      MONGOAC_ERROR_REQUIRE(error);

      CHECK(result);
      CHECK(count_inserted_ids(result) == 3);
   }
}
