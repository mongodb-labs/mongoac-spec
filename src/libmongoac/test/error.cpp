#include <mongoac/error.h>

//

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/bson.h>
#include <mongoac/client.h>
#include <mongoac/collection.h>
#include <mongoac/database.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>
#include <test_util/bson.hh>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

#include <array>
#include <cstdint>

using mongoac::test_util::bson_from_json;
using mongoac::test_util::make_bson_view;
using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::owning_bson;
using mongoac::test_util::to_string;

TEST_CASE("new", "[mongoac][error]")
{
   auto const error = mongoac_error_new();

   CHECK(error != nullptr);

   CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_NONE);
   CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);
   CHECK(to_string(mongoac_error_message(error)).empty());

   CHECK_FALSE(mongoac_error_contains_label(error, "abc"));

   mongoac_error_destroy(error);
}

TEST_CASE("destroy", "[mongoac][error]")
{
   SECTION("null")
   {
      mongoac_error_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("contains_label", "[mongoac][error]")
{
   SECTION("null error")
   {
      CHECK_FALSE(mongoac_error_contains_label(nullptr, "TransientTransactionError"));
   }

   SECTION("valid")
   {
      auto const error = mongoac_error_new();
      REQUIRE(error != nullptr);

      SECTION("null label")
      {
         CHECK_FALSE(mongoac_error_contains_label(error, nullptr));
      }

      SECTION("missing labels")
      {
         CHECK_FALSE(mongoac_error_contains_label(error, "TransientTransactionError"));
         CHECK_FALSE(mongoac_error_contains_label(error, "UnknownTransactionCommitResult"));
      }

      mongoac_error_destroy(error);
   }
}

TEST_CASE("invalid_argument", "[mongoac][error]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client = mongoac_client_new("\x80", error); // Invalid UTF-8.

   CHECK(client == nullptr);
   CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
   CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

   auto const msg = to_string(mongoac_error_message(error));
   CHECK_THAT(msg, Catch::Matchers::StartsWith("invalid argument: "));
   CHECK_THAT(msg, Catch::Matchers::ContainsSubstring("UTF-8"));
}

TEST_CASE("runtime_error", "[mongoac][error]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);
   auto const db =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(client, "admin", nullptr, error), &mongoac_database_destroy);
   auto const cmd = make_owning_ptr(bson_from_json(R"({"ping": 1})"), &bson_destroy);
   auto const future = REQUIRE_MAKE_OWNING_PTR(
      mongoac_database_run_command_async(db, nullptr, make_bson_view(cmd), nullptr, error), &mongoac_future_destroy);
   REQUIRE_FALSE(mongoac_future_is_ready(future));

   auto const view = mongoac_future_get_bson(future, error); // No progress: future is not ready.

   CHECK(view.data == nullptr);
   CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
   CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_RUNTIME_ERROR);

   auto const msg = to_string(mongoac_error_message(error));
   CHECK_THAT(
      msg, Catch::Matchers::StartsWith("runtime error: ") && Catch::Matchers::ContainsSubstring("future is not ready"));
}

TEST_CASE("timeout", "[mongoac][error]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);
   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   auto const db =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(client, "admin", nullptr, error), &mongoac_database_destroy);
   auto const cmd = make_owning_ptr(bson_from_json(R"({"ping": 1})"), &bson_destroy);
   auto const future = REQUIRE_MAKE_OWNING_PTR(
      mongoac_database_run_command_async(db, nullptr, make_bson_view(cmd), nullptr, error), &mongoac_future_destroy);
   REQUIRE_FALSE(mongoac_future_is_ready(future));

   mongoac_runtime_block_on_with_timeout(runtime, future, 0, error); // Instantaneous timeout.

   CHECK(mongoac_error_code(error) != MONGOAC_ERROR_CODE_OK);
   CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
   CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_TIMEOUT);

   auto const msg = to_string(mongoac_error_message(error));
   CHECK_THAT(msg, Catch::Matchers::StartsWith("timeout: "));
}

TEST_CASE("server command error", "[mongoac][error]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);
   auto const db =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(client, "mongoac_error_server_command_error", nullptr, error),
                              &mongoac_database_destroy);

   mongoac_database_drop(db, nullptr, nullptr, error);
   REQUIRE_MONGOAC_OK(error);

   {
      auto const cmd = make_owning_ptr(bson_from_json(R"({"nonexistent": 1})"), &bson_destroy);
      CHECK_FALSE(owning_bson(mongoac_database_run_command(db, nullptr, make_bson_view(cmd), nullptr, error)));
   }

   CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_SERVER);
   CHECK(mongoac_error_code(error) != MONGOAC_ERROR_CODE_OK);
   CHECK(mongoac_error_code(error) == 59); // CommandNotFound
   CHECK_THAT(to_string(mongoac_error_message(error)), Catch::Matchers::ContainsSubstring("no such cmd"));
}

TEST_CASE("server write error", "[mongoac][error]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);
   auto const db =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(client, "mongoac_error_server_write_error", nullptr, error),
                              &mongoac_database_destroy);

   mongoac_database_drop(db, nullptr, nullptr, error);
   REQUIRE_MONGOAC_OK(error);

   auto const coll =
      REQUIRE_MAKE_OWNING_PTR(mongoac_database_get_collection(db, "coll", error), &mongoac_collection_destroy);
   auto const doc = make_owning_ptr(bson_from_json(R"({"_id": 1})"), &bson_destroy);

   {
      auto const result =
         REQUIRE_MAKE_OWNING_BSON(mongoac_collection_insert_one(coll, nullptr, make_bson_view(doc), nullptr, error));

      bson_iter_t iter = {};
      REQUIRE(bson_iter_init_find(&iter, result.bson_ptr(), "insertedId"));
      REQUIRE(bson_iter_type(&iter) == BSON_TYPE_INT32);
      CHECK(bson_iter_int32(&iter) == 1);
   }

   {
      auto const result =
         owning_bson(mongoac_collection_insert_one(coll, nullptr, make_bson_view(doc), nullptr, error));
      CHECK_FALSE(result);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_SERVER);
      CHECK(mongoac_error_code(error) == 11000); // DuplicateKey
      CHECK_THAT(to_string(mongoac_error_message(error)),
                 Catch::Matchers::ContainsSubstring("E11000 duplicate key error"));
   }
}

TEST_CASE("rust", "[mongoac][error]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client = mongoac_client_new("not-a-uri", error); // mongodb::error::ErrorKind::InvalidArgument

   CHECK(client == nullptr);
   CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_RUST);
   CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_UNKNOWN);
   CHECK_THAT(to_string(mongoac_error_message(error)),
              Catch::Matchers::ContainsSubstring("Kind: An invalid argument was provided"));
}

TEST_CASE("bson", "[mongoac][error]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const client =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_new("mongodb://localhost:27017", error), &mongoac_client_destroy);
   auto const runtime = REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   auto const db =
      REQUIRE_MAKE_OWNING_PTR(mongoac_client_get_database(client, "admin", nullptr, error), &mongoac_database_destroy);

   // mongodb::bson::error::ErrorKind::MalformedBytes
   auto const bytes = std::array<std::uint8_t, 2u>{{0xFF, 0xFF}};
   auto const cmd = mongoac_bson_view_t{bytes.data(), bytes.size()};
   CHECK_FALSE(
      make_owning_ptr(mongoac_database_run_command_async(db, nullptr, cmd, nullptr, error), &mongoac_future_destroy));

   CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_BSON);
   CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_UNKNOWN);
   CHECK_THAT(to_string(mongoac_error_message(error)),
              Catch::Matchers::ContainsSubstring("Kind: Malformed BSON bytes"));
}
