#include <mongoac/write_concern.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][write_concern]")
{
   SECTION("default")
   {
      auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);
      REQUIRE(wc != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][write_concern]")
{
   SECTION("null")
   {
      mongoac_write_concern_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_w_majority", "[mongoac][write_concern]")
{
   auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);

   SECTION("null")
   {
      mongoac_write_concern_set_w_majority(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_write_concern_set_w_majority(wc);
      SUCCEED();
   }
}

TEST_CASE("set_w_nodes", "[mongoac][write_concern]")
{
   auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);

   SECTION("null")
   {
      mongoac_write_concern_set_w_nodes(nullptr, 123);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_write_concern_set_w_nodes(wc, 123);
      SUCCEED();
   }
}

TEST_CASE("set_w_custom", "[mongoac][write_concern]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);

   SECTION("null handle")
   {
      mongoac_write_concern_set_w_custom(nullptr, "custom", error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null string")
   {
      mongoac_write_concern_set_w_custom(wc, nullptr, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("valid")
   {
      mongoac_write_concern_set_w_custom(wc, "custom", error);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("set_w_timeout_ms", "[mongoac][write_concern]")
{
   auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);

   SECTION("null")
   {
      mongoac_write_concern_set_w_timeout_ms(nullptr, 123);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_write_concern_set_w_timeout_ms(wc, 123);
      SUCCEED();
   }
}

TEST_CASE("set_journal", "[mongoac][write_concern]")
{
   auto const wc = make_owning_ptr(mongoac_write_concern_new(), &mongoac_write_concern_destroy);

   SECTION("null")
   {
      mongoac_write_concern_set_journal(nullptr, true);
      SUCCEED();
   }

   SECTION("set true")
   {
      mongoac_write_concern_set_journal(wc, true);
      SUCCEED();
   }

   SECTION("set false")
   {
      mongoac_write_concern_set_journal(wc, false);
      SUCCEED();
   }
}
