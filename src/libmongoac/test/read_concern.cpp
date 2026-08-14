#include <mongoac/read_concern.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/error.h>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::to_mongoac;

TEST_CASE("new", "[mongoac][read_concern]")
{
   SECTION("default")
   {
      auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);
      REQUIRE(rc != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][read_concern]")
{
   SECTION("null")
   {
      mongoac_read_concern_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_level_local", "[mongoac][read_concern]")
{
   auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);

   SECTION("null")
   {
      mongoac_read_concern_set_level_local(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_concern_set_level_local(rc);
      SUCCEED();
   }
}

TEST_CASE("set_level_majority", "[mongoac][read_concern]")
{
   auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);

   SECTION("null")
   {
      mongoac_read_concern_set_level_majority(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_concern_set_level_majority(rc);
      SUCCEED();
   }
}

TEST_CASE("set_level_linearizable", "[mongoac][read_concern]")
{
   auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);

   SECTION("null")
   {
      mongoac_read_concern_set_level_linearizable(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_concern_set_level_linearizable(rc);
      SUCCEED();
   }
}

TEST_CASE("set_level_available", "[mongoac][read_concern]")
{
   auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);

   SECTION("null")
   {
      mongoac_read_concern_set_level_available(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_concern_set_level_available(rc);
      SUCCEED();
   }
}

TEST_CASE("set_level_snapshot", "[mongoac][read_concern]")
{
   auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);

   SECTION("null")
   {
      mongoac_read_concern_set_level_snapshot(nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_read_concern_set_level_snapshot(rc);
      SUCCEED();
   }
}

TEST_CASE("set_level_string", "[mongoac][read_concern]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);
   auto const rc = make_owning_ptr(mongoac_read_concern_new(), &mongoac_read_concern_destroy);

   SECTION("null handle")
   {
      mongoac_read_concern_set_level_string(nullptr, to_mongoac("custom"), error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("null string")
   {
      mongoac_read_concern_set_level_string(rc, {}, error);
      REQUIRE_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("custom level")
   {
      mongoac_read_concern_set_level_string(rc, to_mongoac("custom"), error);
      CHECK_MONGOAC_OK(error);
   }
}
