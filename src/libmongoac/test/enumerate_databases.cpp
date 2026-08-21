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
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client = REQUIRE_MAKE_OWNING_PTR(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error),
                                               &mongoac_client_destroy);

   SECTION("client is null")
   {
      auto const future = mongoac_client_list_databases_async(nullptr, nullptr, nullptr, error);

      CHECK(future == nullptr);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }

   SECTION("valid")
   {
      auto const future = REQUIRE_MAKE_OWNING_PTR(mongoac_client_list_databases_async(client, nullptr, nullptr, error),
                                                  &mongoac_future_destroy);
   }

   SECTION("valid with options")
   {
      auto const opts = make_owning_ptr(mongoac_list_databases_options_new(), &mongoac_list_databases_options_destroy);
      mongoac_list_databases_options_set_authorized_databases(opts, true);

      REQUIRE_MAKE_OWNING_PTR(mongoac_client_list_databases_async(client, nullptr, opts, error),
                              &mongoac_future_destroy);
   }
}

TEST_CASE("list_database_names_async", "[mongoac][client]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client = REQUIRE_MAKE_OWNING_PTR(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error),
                                               &mongoac_client_destroy);

   SECTION("null client")
   {
      auto const future = mongoac_client_list_database_names_async(nullptr, nullptr, nullptr, error);

      CHECK(future == nullptr);

      CHECK(mongoac_error_category(error) == MONGOAC_ERROR_CATEGORY_MONGOAC);
      CHECK(mongoac_error_code(error) == MONGOAC_ERROR_CODE_INVALID_ARGUMENT);
   }

   SECTION("null options")
   {
      auto const future = REQUIRE_MAKE_OWNING_PTR(
         mongoac_client_list_database_names_async(client, nullptr, nullptr, error), &mongoac_future_destroy);

      CHECK(future != nullptr);
   }
}

TEST_CASE("list_databases_async returns valid BSON", "[mongoac][client]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   REQUIRE(error);

   auto const client = REQUIRE_MAKE_OWNING_PTR(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error),
                                               &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);

   auto const future = REQUIRE_MAKE_OWNING_PTR(mongoac_client_list_databases_async(client, nullptr, nullptr, error),
                                               &mongoac_future_destroy);

   mongoac_runtime_block_on(runtime, future, error);
   REQUIRE(mongoac_error_code(error) == MONGOAC_ERROR_CODE_OK);

   auto const result = REQUIRE_BSON_VIEW(mongoac_future_get_bson(future, error));

   // Regression test: verify `mongoac_future_get_bson` returns valid bytes.
   auto const json = make_owning_ptr(bson_as_relaxed_extended_json(&result, nullptr), &bson_free);
   REQUIRE(json != nullptr);
   // list_databases returns an indexed array: {"0": { ... }, ...}.
   CHECK_THAT(json.get(), Catch::Matchers::ContainsSubstring("\"0\""));
}
