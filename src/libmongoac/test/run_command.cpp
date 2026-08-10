#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/client.h>
#include <mongoac/cursor.h>
#include <mongoac/database.h>
#include <mongoac/error.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_bson_view;
using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::owning_bson;

TEST_CASE("run_command", "[mongoac][database][run_command]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", nullptr), &mongoac_client_destroy);
   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   auto const db = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(client, "admin", nullptr, nullptr),
                                           &mongoac_database_destroy);

   SECTION("null database")
   {
      auto const cmd = make_owning_ptr(bson_from_json(R"({"ping": 1})"), &bson_destroy);
      auto const reply =
         owning_bson(mongoac_database_run_command(nullptr, nullptr, make_bson_view(cmd), nullptr, error));

      CHECK_FALSE_MONGOAC_OK(error);
      CHECK(reply.data() == nullptr);
   }

   SECTION("async")
   {
      auto const cmd = make_owning_ptr(bson_from_json(R"({"ping": 1})"), &bson_destroy);
      auto const future = make_owning_ptr(
         mongoac_database_run_command_async(db, nullptr, make_bson_view(cmd), nullptr, error), &mongoac_future_destroy);
      REQUIRE_MONGOAC_OK(error);

      mongoac_runtime_block_on(runtime, future, error);
      REQUIRE_MONGOAC_OK(error);

      auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(future, error));

      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, &result, "ok"));
      CHECK(bson_iter_as_double(&iter) == 1.0);
   }

   SECTION("sync")
   {
      auto const cmd = make_owning_ptr(bson_from_json(R"({"ping": 1})"), &bson_destroy);
      auto const reply = owning_bson(mongoac_database_run_command(db, nullptr, make_bson_view(cmd), nullptr, error));

      REQUIRE_MONGOAC_OK(error);
      REQUIRE(reply.data() != nullptr);

      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, reply.bson_ptr(), "ok"));
      CHECK(bson_iter_as_double(&iter) == 1.0);
   }
}

TEST_CASE("run_cursor_command", "[mongoac][database][run_cursor_command]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", nullptr), &mongoac_client_destroy);
   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   auto const db = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(client, "admin", nullptr, nullptr),
                                           &mongoac_database_destroy);

   SECTION("null database")
   {
      auto const cmd = make_owning_ptr(bson_from_json(R"({"listCollections": 1})"), &bson_destroy);
      auto const cursor =
         make_owning_ptr(mongoac_database_run_cursor_command(nullptr, nullptr, make_bson_view(cmd), nullptr, error),
                         &mongoac_cursor_destroy);

      CHECK_FALSE_MONGOAC_OK(error);
      CHECK(cursor == nullptr);
   }

   SECTION("async")
   {
      auto const cmd = make_owning_ptr(bson_from_json(R"({"listCollections": 1})"), &bson_destroy);
      auto const future = REQUIRE_MAKE_OWNING_PTR(
         mongoac_database_run_cursor_command_async(db, nullptr, make_bson_view(cmd), nullptr, error),
         &mongoac_future_destroy);

      mongoac_runtime_block_on(runtime, future, error);
      REQUIRE_MONGOAC_OK(error);

      auto const cursor = make_owning_ptr(mongoac_future_get_cursor(future, error), &mongoac_cursor_destroy);
      REQUIRE_MONGOAC_OK(error);
      REQUIRE(cursor != nullptr);

      CHECK(mongoac_cursor_next(cursor, error));
      CHECK_MONGOAC_OK(error);
   }

   SECTION("sync")
   {
      auto const cmd = make_owning_ptr(bson_from_json(R"({"listCollections": 1})"), &bson_destroy);
      auto const cursor =
         REQUIRE_MAKE_OWNING_PTR(mongoac_database_run_cursor_command(db, nullptr, make_bson_view(cmd), nullptr, error),
                                 &mongoac_cursor_destroy);

      REQUIRE(cursor != nullptr);
      CHECK(mongoac_cursor_next(cursor, error));
      CHECK_MONGOAC_OK(error);
   }
}
