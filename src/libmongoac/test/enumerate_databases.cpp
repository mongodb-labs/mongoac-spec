#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/client.h>
#include <mongoac/error.h>
#include <mongoac/future.h>
#include <mongoac/list_databases_options.h>
#include <mongoac/runtime.h>
#include <test_util/bson.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

#include <string>

using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::to_mongoac;

TEST_CASE("list_databases_async", "[mongoac][client]")
{
   SECTION("client is null")
   {
      auto const error = mongoac_error_new();
      auto const future = mongoac_client_list_databases_async(nullptr, nullptr, nullptr, error);

      CHECK(future == nullptr);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

      mongoac_error_destroy(error);
   }

   SECTION("valid")
   {
      auto const client = mongoac_client_new(to_mongoac("mongodb://localhost:27017"), nullptr);
      REQUIRE(client != nullptr);

      auto const future = mongoac_client_list_databases_async(client, nullptr, nullptr, nullptr);
      CHECK(future != nullptr);

      mongoac_future_destroy(future);
      mongoac_client_destroy(client);
   }

   SECTION("valid with options")
   {
      auto const client = mongoac_client_new(to_mongoac("mongodb://localhost:27017"), nullptr);
      REQUIRE(client != nullptr);

      auto const opts = make_owning_ptr(mongoac_list_databases_options_new(), &mongoac_list_databases_options_destroy);
      mongoac_list_databases_options_set_authorized_databases(opts, true);

      auto const future = mongoac_client_list_databases_async(client, nullptr, opts, nullptr);

      CHECK(future != nullptr);

      mongoac_future_destroy(future);
      mongoac_client_destroy(client);
   }
}

TEST_CASE("list_database_names_async", "[mongoac][client]")
{
   SECTION("null client")
   {
      auto const error = mongoac_error_new();
      auto const future = mongoac_client_list_database_names_async(nullptr, nullptr, nullptr, error);

      CHECK(future == nullptr);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);

      mongoac_error_destroy(error);
   }

   SECTION("null options")
   {
      auto const client = mongoac_client_new(to_mongoac("mongodb://localhost:27017"), nullptr);
      REQUIRE(client != nullptr);

      auto const future = mongoac_client_list_database_names_async(client, nullptr, nullptr, nullptr);

      CHECK(future != nullptr);

      mongoac_future_destroy(future);
      mongoac_client_destroy(client);
   }
}

TEST_CASE("list_databases_async returns valid BSON", "[mongoac][client][live-server]")
{
   auto const error = mongoac_error_new();
   auto const client =
      mongoac_client_new(to_mongoac("mongodb://localhost:27017/?serverSelectionTimeoutMS=2000"), nullptr);
   REQUIRE(client != nullptr);

   auto const runtime = mongoac_client_get_runtime(client);
   REQUIRE(runtime != nullptr);

   auto const future = mongoac_client_list_databases_async(client, nullptr, nullptr, error);
   REQUIRE(future != nullptr);

   mongoac_runtime_block_on(runtime, future, error);
   REQUIRE(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

   auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(future, error));

   // Regression test: verify `mongoac_future_get_bson` returns valid bytes.
   auto const json = make_owning_ptr(bson_as_relaxed_extended_json(&result, nullptr), &bson_free);
   REQUIRE(json != nullptr);
   // list_databases returns an indexed array: {"0": { ... }, ...}.
   CHECK_THAT(json.get(), Catch::Matchers::ContainsSubstring("\"0\""));

   mongoac_future_destroy(future);
   mongoac_runtime_destroy(runtime);
   mongoac_client_destroy(client);
   mongoac_error_destroy(error);
}
