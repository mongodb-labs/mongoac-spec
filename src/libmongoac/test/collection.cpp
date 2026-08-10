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
#include <mongoac/bson.h>
#include <mongoac/client.h>
#include <mongoac/cursor.h>
#include <mongoac/database.h>
#include <mongoac/error.h>
#include <mongoac/find_options.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

#include <array>
#include <cstdint>
#include <optional>

using mongoac::test_util::bson_array_contains_string;
using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::owning_bson;
using mongoac::test_util::to_string;

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
            REQUIRE_MAKE_OWNING_BSON(mongoac_database_list_collection_names(db, nullptr, nullptr, error));
         REQUIRE(bson_empty(names.bson_ptr()));
      }

      mongoac_database_create_collection(db, nullptr, "a", nullptr, error);
      mongoac_database_create_collection(db, nullptr, "b", nullptr, error);
      mongoac_database_create_collection(db, nullptr, "c", nullptr, error);
      {
         auto const names =
            REQUIRE_MAKE_OWNING_BSON(mongoac_database_list_collection_names(db, nullptr, nullptr, error));

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
            REQUIRE_MAKE_OWNING_BSON(mongoac_database_list_collection_names(db, nullptr, nullptr, error));

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
            REQUIRE_MAKE_OWNING_BSON(mongoac_database_list_collection_names(db, nullptr, nullptr, error));

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
            REQUIRE_MAKE_OWNING_BSON(mongoac_database_list_collection_names(db, nullptr, nullptr, error));

         CHECK(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK(bson_array_contains_string(names, "c"));
      }

      mongoac_collection_drop(coll_a, nullptr, nullptr, error);
      mongoac_collection_drop(coll_c, nullptr, nullptr, error);
      {
         auto const names =
            REQUIRE_MAKE_OWNING_BSON(mongoac_database_list_collection_names(db, nullptr, nullptr, error));

         CHECK_FALSE(bson_array_contains_string(names, "a"));
         CHECK_FALSE(bson_array_contains_string(names, "b"));
         CHECK_FALSE(bson_array_contains_string(names, "c"));
      }
   }
}

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
   REQUIRE_MONGOAC_OK(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const doc = make_owning_ptr(bson_from_json(R"({"x": 1})"), &bson_destroy);

   SECTION("async")
   {
      auto const future = REQUIRE_MAKE_OWNING_PTR(
         mongoac_collection_insert_one_async(coll, nullptr, make_bson_view(doc), nullptr, error),
         &mongoac_future_destroy);

      mongoac_runtime_block_on(runtime, future, error);
      REQUIRE_MONGOAC_OK(error);

      auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(future, error));

      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, &result, "insertedId"));
      CHECK(bson_iter_type(&iter) == BSON_TYPE_OID);

      char str[25];
      bson_oid_to_string(bson_iter_oid(&iter), str);
      CHECK_THAT(str, Catch::Matchers::Matches("[0-9a-f]{24}"));
   }

   SECTION("sync")
   {
      auto const result =
         REQUIRE_MAKE_OWNING_BSON(mongoac_collection_insert_one(coll, nullptr, make_bson_view(doc), nullptr, error));

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
   REQUIRE_MONGOAC_OK(error);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_find", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   REQUIRE_MONGOAC_OK(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const filter = make_owning_ptr(bson_from_json(R"({"$or": [{"x": 1}, {"z": 3}]})"), &bson_destroy);
   REQUIRE(filter);

   SECTION("none")
   {
      SECTION("async")
      {
         auto const future = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_find_async(coll, nullptr, make_bson_view(filter), nullptr, error),
            &mongoac_future_destroy);

         mongoac_runtime_block_on(runtime, future, error);
         REQUIRE_MONGOAC_OK(error);

         auto const cursor = REQUIRE_MAKE_OWNING_PTR(mongoac_future_get_cursor(future, error), &mongoac_cursor_destroy);

         CHECK_FALSE(mongoac_cursor_next(cursor, error));
         REQUIRE_MONGOAC_OK(error);
      }

      SECTION("sync")
      {
         auto const cursor = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_find(coll, nullptr, make_bson_view(filter), nullptr, error), &mongoac_cursor_destroy);

         CHECK_FALSE(mongoac_cursor_next(cursor, error));
         REQUIRE_MONGOAC_OK(error);
      }
   }

   SECTION("some")
   {
      {
         auto const x = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 1})"), &bson_destroy);
         auto const y = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"y": 2})"), &bson_destroy);
         auto const z = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"z": 3})"), &bson_destroy);
         auto docs = std::array<mongoac_bson_view_t, 3>{{make_bson_view(x), make_bson_view(y), make_bson_view(z)}};

         REQUIRE_MAKE_OWNING_BSON(
            mongoac_collection_insert_many(coll, nullptr, docs.data(), docs.size(), nullptr, error));
      }

      // Sorting by _id ascending returns the two matches in insertion order;
      // since x is inserted before z, the {x: 1} document precedes {z: 3}.
      auto const sort_bson = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"sort": {"_id": 1}})"), &bson_destroy);
      auto const options = REQUIRE_MAKE_OWNING_PTR(mongoac_find_options_new_from_bson(make_bson_view(sort_bson), error),
                                                   &mongoac_find_options_destroy);

      auto const check_two_results = [&](mongoac_cursor_t const *cursor) {
         bson_iter_t iter = {};

         REQUIRE(mongoac_cursor_next(cursor, error));
         REQUIRE_MONGOAC_OK(error);
         {
            auto const first = REQUIRE_BSON_VIEW(mongoac_cursor_current(cursor, error));
            REQUIRE(bson_iter_init_find(&iter, &first, "x"));
            REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT32);
            CHECK(bson_iter_int32(&iter) == 1);
         }

         REQUIRE(mongoac_cursor_next(cursor, error));
         REQUIRE_MONGOAC_OK(error);
         {
            auto const second = REQUIRE_BSON_VIEW(mongoac_cursor_current(cursor, error));
            REQUIRE(bson_iter_init_find(&iter, &second, "z"));
            REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT32);
            CHECK(bson_iter_int32(&iter) == 3);
         }

         CHECK_FALSE(mongoac_cursor_next(cursor, error));
         REQUIRE_MONGOAC_OK(error);
      };

      SECTION("async")
      {
         auto const future = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_find_async(coll, nullptr, make_bson_view(filter), options, error),
            &mongoac_future_destroy);

         mongoac_runtime_block_on(runtime, future, error);
         REQUIRE_MONGOAC_OK(error);

         auto const cursor = REQUIRE_MAKE_OWNING_PTR(mongoac_future_get_cursor(future, error), &mongoac_cursor_destroy);

         check_two_results(cursor.get());
      }

      SECTION("sync")
      {
         auto const cursor = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_find(coll, nullptr, make_bson_view(filter), options, error), &mongoac_cursor_destroy);

         check_two_results(cursor.get());
      }
   }
}

TEST_CASE("find_one", "[mongoac][collection]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_find_one", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   REQUIRE_MONGOAC_OK(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const x1 = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 1})"), &bson_destroy);
   auto const x2 = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 2})"), &bson_destroy);

   REQUIRE_MAKE_OWNING_BSON(mongoac_collection_insert_one(coll, nullptr, make_bson_view(x1), nullptr, error));

   SECTION("async")
   {
      auto const f1 = REQUIRE_MAKE_OWNING_PTR(
         mongoac_collection_find_one_async(coll, nullptr, make_bson_view(x1), nullptr, error), &mongoac_future_destroy);
      auto const f2 = REQUIRE_MAKE_OWNING_PTR(
         mongoac_collection_find_one_async(coll, nullptr, make_bson_view(x2), nullptr, error), &mongoac_future_destroy);

      {
         auto const futures = std::array<mongoac_future_t const *, 2u>{{f1, f2}};
         mongoac_runtime_block_on_all(runtime, futures.data(), futures.size(), error);
      }

      auto const r1 = REQUIRE_MAKE_OWNING_BSON(mongoac_future_get_optional_bson(f1, error));
      auto const r2 = REQUIRE_MAKE_OWNING_BSON(mongoac_future_get_optional_bson(f2, error));

      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, r1, "x"));
      REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT32);
      CHECK(bson_iter_int32(&iter) == 1);

      CHECK_FALSE(r2);
   }

   SECTION("sync")
   {
      {
         auto const result =
            REQUIRE_MAKE_OWNING_BSON(mongoac_collection_find_one(coll, nullptr, make_bson_view(x1), nullptr, error));
         REQUIRE(result);

         bson_iter_t iter = {};
         REQUIRE(bson_iter_init_find(&iter, result, "x"));
         REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT32);
         CHECK(bson_iter_int32(&iter) == 1);
      }

      {
         auto const result =
            REQUIRE_MAKE_OWNING_BSON(mongoac_collection_find_one(coll, nullptr, make_bson_view(x2), nullptr, error));
         CHECK_FALSE(result);
      }
   }
}


TEST_CASE("insert_many", "[mongoac][collection]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_insert_many", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   REQUIRE_MONGOAC_OK(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const x = REQUIRE_MAKE_OWNING_PTR(BCON_NEW("x", BCON_INT32(1)), &bson_destroy);
   auto const y = REQUIRE_MAKE_OWNING_PTR(BCON_NEW("y", BCON_INT32(2)), &bson_destroy);
   auto const z = REQUIRE_MAKE_OWNING_PTR(BCON_NEW("z", BCON_INT32(3)), &bson_destroy);

   auto docs = std::array<mongoac_bson_view_t, 3u>{{make_bson_view(x), make_bson_view(y), make_bson_view(z)}};

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

      mongoac_runtime_block_on(runtime, future, error);
      REQUIRE_MONGOAC_OK(error);

      auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(future, error));

      CHECK(count_inserted_ids(&result) == 3);
   }

   SECTION("sync")
   {
      auto const result = REQUIRE_MAKE_OWNING_BSON(
         mongoac_collection_insert_many(coll, nullptr, docs.data(), docs.size(), nullptr, error));

      CHECK(result);
      CHECK(count_inserted_ids(result) == 3);
   }
}

TEST_CASE("delete", "[mongoac][collection]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_delete", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   REQUIRE_MONGOAC_OK(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const count_results = [&](mongoac_bson_view_t filter) -> int {
      auto const cursor =
         make_owning_ptr(mongoac_collection_find(coll, nullptr, filter, nullptr, error), &mongoac_cursor_destroy);

      if (mongoac_error_code(error) != MONGOAC_ERROR_CODE_OK) {
         UNSCOPED_CAPTURE(mongoac_error_message(error));
         return 0;
      }

      int n = 0;
      while (mongoac_cursor_next(cursor, error) && mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK) {
         ++n;
      }
      return n;
   };

   auto const get_deleted_count = [](bson_t const &result) -> std::int64_t {
      bson_iter_t iter = {};
      if (bson_iter_init_find(&iter, &result, "deletedCount") && bson_iter_type(&iter) == BSON_TYPE_INT64) {
         return bson_iter_int64(&iter);
      }
      return 0;
   };

   auto const x1 = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 1})"), &bson_destroy);
   auto const x2 = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 2})"), &bson_destroy); // Inserted twice.

   // Setup.
   {
      auto const docs = std::array<mongoac_bson_view_t, 3u>{{
         make_bson_view(x1),
         make_bson_view(x2),
         make_bson_view(x2), // Inserted twice.
      }};

      REQUIRE_MAKE_OWNING_BSON(mongoac_collection_insert_many(coll, nullptr, docs.data(), docs.size(), nullptr, error));
   }

   SECTION("async")
   {
      CHECK(count_results(make_bson_view(x1)) == 1);
      CHECK(count_results(make_bson_view(x2)) == 2);

      {
         auto const del = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_delete_one_async(coll, nullptr, make_bson_view(x1), nullptr, error),
            &mongoac_future_destroy);

         mongoac_runtime_block_on(runtime, del, error);
         REQUIRE_MONGOAC_OK(error);

         auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(del, error));
         CHECK(get_deleted_count(result) == 1);
      }

      CHECK(count_results(make_bson_view(x1)) == 0);
      CHECK(count_results(make_bson_view(x2)) == 2);

      {
         auto const del = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_delete_many_async(coll, nullptr, make_bson_view(x2), nullptr, error),
            &mongoac_future_destroy);
         mongoac_runtime_block_on(runtime, del, error);
         REQUIRE_MONGOAC_OK(error);
         auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(del, error));
         CHECK(get_deleted_count(result) == 2);
      }

      CHECK(count_results(make_bson_view(x1)) == 0);
      CHECK(count_results(make_bson_view(x2)) == 0);
   }

   SECTION("sync")
   {
      CHECK(count_results(make_bson_view(x1)) == 1);
      CHECK(count_results(make_bson_view(x2)) == 2);

      {
         auto const result =
            REQUIRE_MAKE_OWNING_BSON(mongoac_collection_delete_one(coll, nullptr, make_bson_view(x1), nullptr, error));
         CHECK(result);
         CHECK(get_deleted_count(result) == 1);
      }

      CHECK(count_results(make_bson_view(x1)) == 0);
      CHECK(count_results(make_bson_view(x2)) == 2);

      {
         auto const result =
            REQUIRE_MAKE_OWNING_BSON(mongoac_collection_delete_many(coll, nullptr, make_bson_view(x2), nullptr, error));
         CHECK(result);
         CHECK(get_deleted_count(result) == 2);
      }

      CHECK(count_results(make_bson_view(x1)) == 0);
      CHECK(count_results(make_bson_view(x2)) == 0);
   }
}

TEST_CASE("replace_one", "[mongoac][collection]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_replace_one", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   REQUIRE_MONGOAC_OK(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const check_result = [](bson_t const *result, int64_t matched, int64_t modified) {
      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, result, "matchedCount"));
      REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT64);
      CHECK(bson_iter_int64(&iter) == matched);
      REQUIRE(bson_iter_init_find(&iter, result, "modifiedCount"));
      REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT64);
      CHECK(bson_iter_int64(&iter) == modified);
   };

   auto const x1 = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 1})"), &bson_destroy);
   auto const x2 = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 2})"), &bson_destroy);
   REQUIRE_MAKE_OWNING_BSON(mongoac_collection_insert_one(coll, nullptr, make_bson_view(x1), nullptr, error));

   SECTION("async")
   {
      auto const future = REQUIRE_MAKE_OWNING_PTR(
         mongoac_collection_replace_one_async(coll, nullptr, make_bson_view(x1), make_bson_view(x2), nullptr, error),
         &mongoac_future_destroy);
      mongoac_runtime_block_on(runtime, future, error);
      REQUIRE_MONGOAC_OK(error);
      auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(future, error));
      check_result(&result, 1, 1);
      CHECK_FALSE(owning_bson(mongoac_collection_find_one(coll, nullptr, make_bson_view(x1), nullptr, error)));
      CHECK(owning_bson(mongoac_collection_find_one(coll, nullptr, make_bson_view(x2), nullptr, error)));
   }

   SECTION("sync")
   {
      auto const result = REQUIRE_MAKE_OWNING_BSON(
         mongoac_collection_replace_one(coll, nullptr, make_bson_view(x1), make_bson_view(x2), nullptr, error));
      CHECK(result);
      check_result(result, 1, 1);
      CHECK_FALSE(owning_bson(mongoac_collection_find_one(coll, nullptr, make_bson_view(x1), nullptr, error)));
      CHECK(owning_bson(mongoac_collection_find_one(coll, nullptr, make_bson_view(x2), nullptr, error)));
   }
}

TEST_CASE("update", "[mongoac][collection]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);

   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const db = REQUIRE_MAKE_OWNING_PTR(
      mongoac_client_get_database(client, "mongoac_collection_update", nullptr, error), &mongoac_database_destroy);

   // Clean test state.
   mongoac_database_drop(db, nullptr, nullptr, error);
   REQUIRE_MONGOAC_OK(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);

   auto const get_matched_count = [](bson_t const &result) -> int64_t {
      bson_iter_t iter = {};
      if (bson_iter_init_find(&iter, &result, "matchedCount") && bson_iter_type(&iter) == BSON_TYPE_INT64) {
         return bson_iter_int64(&iter);
      }
      return 0;
   };

   auto const get_modified_count = [](bson_t const &result) -> int64_t {
      bson_iter_t iter = {};
      if (bson_iter_init_find(&iter, &result, "modifiedCount") && bson_iter_type(&iter) == BSON_TYPE_INT64) {
         return bson_iter_int64(&iter);
      }
      return 0;
   };

   auto const count = [&](int32_t n) -> int {
      auto const cursor =
         REQUIRE_MAKE_OWNING_PTR(mongoac_collection_find(coll, nullptr, {}, nullptr, error), &mongoac_cursor_destroy);
      int match = 0;
      while (mongoac_cursor_next(cursor, error)) {
         REQUIRE_MONGOAC_OK(error);
         auto const doc = REQUIRE_BSON_VIEW(mongoac_cursor_current(cursor, error));
         bson_iter_t iter = {};
         REQUIRE(bson_iter_init_find(&iter, &doc, "x"));
         if (bson_iter_int32(&iter) == n) {
            ++match;
         }
      }
      REQUIRE_MONGOAC_OK(error);
      return match;
   };

   {
      auto const x1 = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"x": 1})"), &bson_destroy);
      auto const docs = std::array<mongoac_bson_view_t, 2u>{{make_bson_view(x1), make_bson_view(x1)}};
      REQUIRE_MAKE_OWNING_BSON(mongoac_collection_insert_many(coll, nullptr, docs.data(), docs.size(), nullptr, error));
   }

   SECTION("async")
   {
      {
         auto const update = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"$set": {"x": 2}})"), &bson_destroy);
         auto const future = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_update_one_async(coll, nullptr, {}, make_bson_view(update), nullptr, error),
            &mongoac_future_destroy);
         mongoac_runtime_block_on(runtime, future, error);
         REQUIRE_MONGOAC_OK(error);
         auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(future, error));
         CHECK(get_matched_count(result) == 1);
         CHECK(get_modified_count(result) == 1);
      }

      CHECK(count(1) == 1);
      CHECK(count(2) == 1);
      CHECK(count(3) == 0);

      {
         auto const upd = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"$set": {"x": 3}})"), &bson_destroy);
         auto const future = REQUIRE_MAKE_OWNING_PTR(
            mongoac_collection_update_many_async(coll, nullptr, {}, make_bson_view(upd), nullptr, error),
            &mongoac_future_destroy);
         mongoac_runtime_block_on(runtime, future, error);
         REQUIRE_MONGOAC_OK(error);
         auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(future, error));
         CHECK(get_matched_count(result) == 2);
         CHECK(get_modified_count(result) == 2);
      }

      CHECK(count(1) == 0);
      CHECK(count(2) == 0);
      CHECK(count(3) == 2);
   }

   SECTION("sync")
   {
      {
         auto const upd = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"$set": {"x": 2}})"), &bson_destroy);
         auto const result = REQUIRE_MAKE_OWNING_BSON(
            mongoac_collection_update_one(coll, nullptr, {}, make_bson_view(upd), nullptr, error));
         CHECK(result);
         CHECK(get_matched_count(result) == 1);
         CHECK(get_modified_count(result) == 1);
      }

      CHECK(count(1) == 1);
      CHECK(count(2) == 1);
      CHECK(count(3) == 0);

      {
         auto const upd = REQUIRE_MAKE_OWNING_PTR(bson_from_json(R"({"$set": {"x": 3}})"), &bson_destroy);
         auto const result = REQUIRE_MAKE_OWNING_BSON(
            mongoac_collection_update_many(coll, nullptr, {}, make_bson_view(upd), nullptr, error));
         CHECK(result);
         CHECK(get_matched_count(result) == 2);
         CHECK(get_modified_count(result) == 2);
      }

      CHECK(count(1) == 0);
      CHECK(count(2) == 0);
      CHECK(count(3) == 2);
   }
}
