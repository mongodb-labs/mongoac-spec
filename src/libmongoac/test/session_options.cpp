#include <mongoac/session_options.h>

#include <catch2/catch_test_macros.hpp>
#include <mongoac/transaction_options.h>
#include <test_util/owning_ptr.hh>

using mongoac::test_util::make_owning_ptr;

TEST_CASE("new", "[mongoac][session_options]")
{
   SECTION("default")
   {
      auto const opts = make_owning_ptr(mongoac_session_options_new(), &mongoac_session_options_destroy);
      REQUIRE(opts != nullptr);
   }
}

TEST_CASE("destroy", "[mongoac][session_options]")
{
   SECTION("null")
   {
      mongoac_session_options_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("set_causal_consistency", "[mongoac][session_options]")
{
   auto const opts = make_owning_ptr(mongoac_session_options_new(), &mongoac_session_options_destroy);

   SECTION("null")
   {
      mongoac_session_options_set_causal_consistency(nullptr, true);
      SUCCEED();
   }

   SECTION("true")
   {
      mongoac_session_options_set_causal_consistency(opts, true);
      SUCCEED();
   }

   SECTION("false")
   {
      mongoac_session_options_set_causal_consistency(opts, false);
      SUCCEED();
   }
}

TEST_CASE("set_snapshot", "[mongoac][session_options]")
{
   auto const opts = make_owning_ptr(mongoac_session_options_new(), &mongoac_session_options_destroy);

   SECTION("null")
   {
      mongoac_session_options_set_snapshot(nullptr, true);
      SUCCEED();
   }

   SECTION("true")
   {
      mongoac_session_options_set_snapshot(opts, true);
      SUCCEED();
   }

   SECTION("false")
   {
      mongoac_session_options_set_snapshot(opts, false);
      SUCCEED();
   }
}

TEST_CASE("set_snapshot_time", "[mongoac][session_options]")
{
   auto const opts = make_owning_ptr(mongoac_session_options_new(), &mongoac_session_options_destroy);

   SECTION("null")
   {
      mongoac_session_options_set_snapshot_time(nullptr, 123, 456);
      SUCCEED();
   }

   SECTION("valid")
   {
      mongoac_session_options_set_snapshot_time(opts, 123, 456);
      SUCCEED();
   }
}

TEST_CASE("set_default_transaction_options", "[mongoac][session_options]")
{
   auto const opts = make_owning_ptr(mongoac_session_options_new(), &mongoac_session_options_destroy);

   SECTION("null handle")
   {
      mongoac_session_options_set_default_transaction_options(nullptr, nullptr);
      SUCCEED();
   }

   SECTION("null clears")
   {
      auto const txn_opts = make_owning_ptr(mongoac_transaction_options_new(), &mongoac_transaction_options_destroy);
      mongoac_session_options_set_default_transaction_options(opts, txn_opts);
      mongoac_session_options_set_default_transaction_options(opts, nullptr);
      SUCCEED();
   }

   SECTION("valid")
   {
      auto const txn_opts = make_owning_ptr(mongoac_transaction_options_new(), &mongoac_transaction_options_destroy);
      mongoac_session_options_set_default_transaction_options(opts, txn_opts);
      SUCCEED();
   }
}
