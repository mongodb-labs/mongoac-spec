#include <mongoac/runtime.h>

//

#include <catch2/catch_test_macros.hpp>
#include <mongoac/client.h>
#include <mongoac/error.h>
#include <mongoac/future-fwd.h>
#include <mongoac/future.h>
#include <test_util/error.hh>
#include <test_util/owning_ptr.hh>
#include <test_util/string.hh>

using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::to_mongoac;

TEST_CASE("make_progress", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_make_progress(nullptr);
      SUCCEED();
   }
}

TEST_CASE("make_progress_with_timeout", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_make_progress_with_timeout(nullptr, 0, nullptr);
      SUCCEED();
   }
}

TEST_CASE("make_progress_for", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_make_progress_for(nullptr, 0);
      SUCCEED();
   }
}

TEST_CASE("runtime destroy", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_destroy(nullptr);
      SUCCEED();
   }
}

TEST_CASE("block_on_any", "[mongoac][runtime]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   auto const client = REQUIRE_MAKE_OWNING_PTR(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error),
                                               &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   REQUIRE(runtime != nullptr);

   SECTION("null")
   {
      CHECK(mongoac_runtime_block_on_any(nullptr, nullptr, 0, nullptr) == nullptr);
   }

   SECTION("empty array")
   {
      mongoac_future_t *const futures[] = {nullptr};

      CHECK(mongoac_runtime_block_on_any(runtime, futures, 0, error) == nullptr);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("null element")
   {
      mongoac_future_t *const futures[] = {nullptr};

      CHECK(mongoac_runtime_block_on_any(runtime, futures, 1, error) == nullptr);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("incompatible runtime")
   {
      auto const other = REQUIRE_MAKE_OWNING_PTR(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error),
                                                 &mongoac_client_destroy);

      auto const future = REQUIRE_MAKE_OWNING_PTR(mongoac_client_shutdown_async(other, error), &mongoac_future_destroy);

      mongoac_future_t *const futures[] = {future.get()};

      CHECK(mongoac_runtime_block_on_any(runtime, futures, 1, error) == nullptr);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }

   SECTION("ready element skips null element after")
   {
      auto const ready = REQUIRE_MAKE_OWNING_PTR(mongoac_client_shutdown_async(client, error), &mongoac_future_destroy);
      mongoac_runtime_block_on(runtime, ready, error);
      REQUIRE_MONGOAC_OK(error);

      mongoac_future_t *const futures[] = {ready.get(), nullptr};

      auto const ret = mongoac_runtime_block_on_any(runtime, futures, 2, error);
      CHECK(ret == &futures[0]);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("ready element skips foreign element after")
   {
      auto const other_client = REQUIRE_MAKE_OWNING_PTR(
         mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error), &mongoac_client_destroy);

      auto const foreign =
         REQUIRE_MAKE_OWNING_PTR(mongoac_client_shutdown_async(other_client, error), &mongoac_future_destroy);

      auto const ready = REQUIRE_MAKE_OWNING_PTR(mongoac_client_shutdown_async(client, error), &mongoac_future_destroy);
      mongoac_runtime_block_on(runtime, ready, error);
      REQUIRE_MONGOAC_OK(error);

      mongoac_future_t *const futures[] = {ready.get(), foreign.get()};

      auto const ret = mongoac_runtime_block_on_any(runtime, futures, 2, error);
      CHECK(ret == &futures[0]);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("ready element skips foreign ready element before")
   {
      auto const other_client = REQUIRE_MAKE_OWNING_PTR(
         mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error), &mongoac_client_destroy);

      auto const other_runtime = make_owning_ptr(mongoac_client_get_runtime(other_client), &mongoac_runtime_destroy);
      REQUIRE(other_runtime != nullptr);

      auto const foreign =
         REQUIRE_MAKE_OWNING_PTR(mongoac_client_shutdown_async(other_client, error), &mongoac_future_destroy);
      mongoac_runtime_block_on(other_runtime, foreign, error);
      REQUIRE_MONGOAC_OK(error);

      auto const ready = REQUIRE_MAKE_OWNING_PTR(mongoac_client_shutdown_async(client, error), &mongoac_future_destroy);
      mongoac_runtime_block_on(runtime, ready, error);
      REQUIRE_MONGOAC_OK(error);

      mongoac_future_t *const futures[] = {foreign.get(), ready.get()};

      auto const ret = mongoac_runtime_block_on_any(runtime, futures, 2, error);
      CHECK(ret == &futures[1]);
      CHECK_MONGOAC_OK(error);
   }
}

TEST_CASE("block_on", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_block_on(nullptr, nullptr, nullptr);
      SUCCEED();
   }
}

TEST_CASE("block_on_all", "[mongoac][runtime]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   auto const client = REQUIRE_MAKE_OWNING_PTR(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error),
                                               &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   REQUIRE(runtime != nullptr);

   SECTION("null")
   {
      mongoac_runtime_block_on_all(nullptr, nullptr, 0, nullptr);
      SUCCEED();
   }

   SECTION("null element")
   {
      mongoac_future_t *const futures[] = {nullptr};

      mongoac_runtime_block_on_all(runtime, futures, 1, error);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }
}

TEST_CASE("block_on_with_timeout", "[mongoac][runtime]")
{
   SECTION("null")
   {
      mongoac_runtime_block_on_with_timeout(nullptr, nullptr, 0, nullptr);
      SUCCEED();
   }
}

TEST_CASE("block_on_any_with_timeout", "[mongoac][runtime]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   auto const client = REQUIRE_MAKE_OWNING_PTR(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error),
                                               &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   REQUIRE(runtime != nullptr);

   SECTION("null")
   {
      CHECK(mongoac_runtime_block_on_any_with_timeout(nullptr, nullptr, 0, 0, nullptr) == nullptr);
   }

   SECTION("empty array")
   {
      mongoac_future_t *const futures[] = {nullptr};

      CHECK(mongoac_runtime_block_on_any_with_timeout(runtime, futures, 0, 1000, error) == nullptr);
      CHECK_MONGOAC_OK(error);
   }

   SECTION("null element")
   {
      mongoac_future_t *const futures[] = {nullptr};

      CHECK(mongoac_runtime_block_on_any_with_timeout(runtime, futures, 1, 1000, error) == nullptr);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }
}

TEST_CASE("block_on_all_with_timeout", "[mongoac][runtime]")
{
   auto const error = make_owning_ptr(mongoac_error_new(), &mongoac_error_destroy);

   auto const client = REQUIRE_MAKE_OWNING_PTR(mongoac_client_new(to_mongoac("mongodb://localhost:27017"), error),
                                               &mongoac_client_destroy);

   auto const runtime = make_owning_ptr(mongoac_client_get_runtime(client), &mongoac_runtime_destroy);
   REQUIRE(runtime != nullptr);

   SECTION("null")
   {
      mongoac_runtime_block_on_all_with_timeout(nullptr, nullptr, 0, 0, nullptr);
      SUCCEED();
   }

   SECTION("null element")
   {
      mongoac_future_t *const futures[] = {nullptr};

      mongoac_runtime_block_on_all_with_timeout(runtime, futures, 1, 1000, error);
      CHECK_MONGOAC_INVALID_ARGUMENT(error);
   }
}

TEST_CASE("lifetime", "[mongoac][runtime]")
{
   auto const client = mongoac_client_new(to_mongoac("mongodb://localhost:27017"), nullptr);
   REQUIRE(client != nullptr);

   auto const runtime = mongoac_client_get_runtime(client);
   REQUIRE(runtime != nullptr);

   mongoac_client_destroy(client); // RuntimeT may outlive ClientT.

   mongoac_runtime_destroy(runtime);
}
