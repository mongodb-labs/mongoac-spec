#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/client.h>
#include <mongoac/error.h>
#include <mongoac/future.h>
#include <mongoac/runtime.h>

#include <string>

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
      auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);
      REQUIRE(client != nullptr);

      auto const future = mongoac_client_list_databases_async(client, nullptr, nullptr, nullptr);
      CHECK(future != nullptr);

      mongoac_future_destroy(future);
      mongoac_client_destroy(client);
   }

   SECTION("valid with options")
   {
      auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);
      REQUIRE(client != nullptr);

      bson_t opts_bson;
      bson_init(&opts_bson);
      bson_append_bool(&opts_bson, "authorizedDatabases", -1, true);

      auto const future = mongoac_client_list_databases_async(client, nullptr, &opts_bson, nullptr);

      CHECK(future != nullptr);

      bson_destroy(&opts_bson);
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
      auto const client = mongoac_client_new("mongodb://localhost:27017", nullptr);
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
   auto const client = mongoac_client_new("mongodb://localhost:27017/?serverSelectionTimeoutMS=2000", nullptr);
   REQUIRE(client != nullptr);

   auto const runtime = mongoac_client_get_runtime(client);
   REQUIRE(runtime != nullptr);

   auto const future = mongoac_client_list_databases_async(client, nullptr, nullptr, error);
   REQUIRE(future != nullptr);

   mongoac_runtime_block_on(runtime, future, error);
   REQUIRE(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

   bson_t *const result = mongoac_future_get_bson(future, error);
   REQUIRE(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);
   REQUIRE(result != nullptr);

   // Regression test: verify `mongoac_future_get_bson` returns a valid pointer.
   char *const json = bson_as_relaxed_extended_json(result, nullptr);
   REQUIRE(json != nullptr);
   // list_databases returns an indexed array: {"0": { ... }, ...}.
   CHECK_THAT(std::string(json), Catch::Matchers::ContainsSubstring("\"0\""));
   bson_free(json);

   bson_destroy(result);
   mongoac_future_destroy(future);
   mongoac_runtime_destroy(runtime);
   mongoac_client_destroy(client);
   mongoac_error_destroy(error);
}
