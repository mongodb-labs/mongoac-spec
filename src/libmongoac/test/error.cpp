#include <mongoac/error.h>

//

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>

TEST_CASE("new", "[mongoac][error]")
{
   auto const error = mongoac_error_new();

   CHECK(error != nullptr);

   CHECK(mongoac_error_category(error) == 0);
   CHECK(mongoac_error_code(error) == 0);
   CHECK(mongoac_error_message(error) != nullptr);

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

TEST_CASE("has_label", "[mongoac][error]")
{
   SECTION("null error")
   {
      CHECK_FALSE(mongoac_error_has_label(nullptr, "TransientTransactionError"));
   }

   SECTION("valid")
   {
      auto const error = mongoac_error_new();
      REQUIRE(error != nullptr);

      SECTION("null label")
      {
         CHECK_FALSE(mongoac_error_has_label(error, nullptr));
      }

      SECTION("missing labels")
      {
         CHECK_FALSE(mongoac_error_has_label(error, "TransientTransactionError"));
         CHECK_FALSE(mongoac_error_has_label(error, "UnknownTransactionCommitResult"));
      }

      mongoac_error_destroy(error);
   }
}
