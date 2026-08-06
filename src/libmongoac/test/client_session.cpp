#include <mongoac/client_session.h>

//

#include <bson/bson.h>

#include <catch2/catch_test_macros.hpp>
#include <catch2/matchers/catch_matchers_string.hpp>
#include <mongoac/bson.h>
#include <mongoac/client.h>
#include <mongoac/client_options.h>
#include <mongoac/cursor.h>
#include <mongoac/database.h>
#include <test_util/bson.hh>
#include <test_util/owning_ptr.hh>

#include <cstdint>
#include <cstring>

using mongoac::test_util::make_owning_ptr;
using mongoac::test_util::owning_bson;

namespace
{

struct lsid {
   static constexpr std::size_t len = 16; // UUID

   bson_subtype_t subtype;
   uint8_t data[len]; // UUID storage — owned copy, not a borrowed pointer.

   friend bool
   operator==(lsid const &a, lsid const &b)
   {
      return a.subtype == b.subtype && std::memcmp(a.data, b.data, len) == 0;
   }

   friend bool
   operator!=(lsid const &a, lsid const &b)
   {
      return !(a == b);
   }
};

const char *
get_command_name(bson_t const &event)
{
   bson_iter_t iter = {};

   if (!bson_iter_init(&iter, &event) || !bson_iter_find_descendant(&iter, "commandName", &iter) ||
       !BSON_ITER_HOLDS_UTF8(&iter)) {
      return nullptr;
   }

   return bson_iter_utf8(&iter, nullptr);
}

const char *
get_event_type(bson_t const &event)
{
   bson_iter_t iter = {};

   {
      auto const json = make_owning_ptr(bson_as_canonical_extended_json(&event, nullptr), &bson_free);
      UNSCOPED_INFO(json);
   }

   if (bson_iter_init_find(&iter, &event, "commandName")) {
      if (bson_iter_init_find(&iter, &event, "reply")) {
         return "CommandSucceededEvent";
      }

      if (bson_iter_init_find(&iter, &event, "failure")) {
         return "CommandFailedEvent";
      }

      return "CommandStartedEvent";
   }

   return "unknown";
}

bool
get_lsid(bson_t const &event, lsid *out)
{
   bson_iter_t iter = {};

   if (!bson_iter_init(&iter, &event) || !bson_iter_find_descendant(&iter, "command.lsid.id", &iter) ||
       !BSON_ITER_HOLDS_BINARY(&iter)) {
      return false;
   }

   const uint8_t *data = {};
   uint32_t len = {};
   bson_iter_binary(&iter, &out->subtype, &len, &data);

   if (out->subtype != BSON_SUBTYPE_UUID || len != lsid::len) {
      return false;
   }

   std::memcpy(out->data, data, lsid::len);
   return true;
}

} // namespace

TEST_CASE("sessions", "[mongoac][client_session]")
{
   auto const opts = make_owning_ptr(mongoac_client_options_new(), &mongoac_client_options_destroy);
   mongoac_client_options_set_capture_command_events(opts, true);

   auto const client = make_owning_ptr(mongoac_client_new_with_options(opts, nullptr), &mongoac_client_destroy);

   auto const db = make_owning_ptr(mongoac_client_get_database(client, "mongoac_session_lsid", nullptr, nullptr),
                                   &mongoac_database_destroy);

   // Clean test state.
   {
      mongoac_database_drop(db, nullptr, nullptr, nullptr);
      auto const names = owning_bson(mongoac_database_list_collection_names(db, nullptr, nullptr, nullptr));
      REQUIRE(names);
      mongoac_client_clear_command_events(client, mongoac_client_count_command_events(client));
   }

   auto const assert_session_events = [&client](bool expect_equal) {
      size_t const count = mongoac_client_count_command_events(client);
      REQUIRE(count == 4); // CommandStarted + CommandSucceeded for each listCollections operation.

      auto const e0 = owning_bson(mongoac_client_get_command_event(client, 0));
      auto const e1 = owning_bson(mongoac_client_get_command_event(client, 1));
      auto const e2 = owning_bson(mongoac_client_get_command_event(client, 2));
      auto const e3 = owning_bson(mongoac_client_get_command_event(client, 3));

      REQUIRE(e0);
      REQUIRE(e1);
      REQUIRE(e2);
      REQUIRE(e3);

      CHECK_THAT(get_command_name(e0), Catch::Matchers::Equals("listCollections"));
      CHECK_THAT(get_command_name(e1), Catch::Matchers::Equals("listCollections"));
      CHECK_THAT(get_command_name(e2), Catch::Matchers::Equals("listCollections"));
      CHECK_THAT(get_command_name(e3), Catch::Matchers::Equals("listCollections"));

      CHECK_THAT(get_event_type(e0), Catch::Matchers::Equals("CommandStartedEvent"));
      CHECK_THAT(get_event_type(e1), Catch::Matchers::Equals("CommandSucceededEvent"));
      CHECK_THAT(get_event_type(e2), Catch::Matchers::Equals("CommandStartedEvent"));
      CHECK_THAT(get_event_type(e3), Catch::Matchers::Equals("CommandSucceededEvent"));

      lsid lsid_a = {};
      lsid lsid_b = {};

      REQUIRE(get_lsid(e0, &lsid_a));
      REQUIRE(get_lsid(e2, &lsid_b));

      if (expect_equal) {
         CHECK(lsid_a == lsid_b);
      } else {
         CHECK(lsid_a != lsid_b);
      }
   };

   SECTION("different sessions")
   {
      auto const session_a =
         make_owning_ptr(mongoac_client_start_session(client, nullptr, nullptr), &mongoac_client_session_destroy);
      auto const session_b =
         make_owning_ptr(mongoac_client_start_session(client, nullptr, nullptr), &mongoac_client_session_destroy);

      {
         auto const cursor = make_owning_ptr(mongoac_database_list_collections(db, session_a, nullptr, nullptr),
                                             &mongoac_cursor_destroy);
         CHECK_FALSE(mongoac_cursor_next(cursor, nullptr)); // No collections in the database.
      }

      {
         auto const cursor = make_owning_ptr(mongoac_database_list_collections(db, session_b, nullptr, nullptr),
                                             &mongoac_cursor_destroy);
         CHECK_FALSE(mongoac_cursor_next(cursor, nullptr)); // No collections in the database.
      }

      assert_session_events(false);
   }

   SECTION("same session")
   {
      auto const session =
         make_owning_ptr(mongoac_client_start_session(client, nullptr, nullptr), &mongoac_client_session_destroy);

      {
         auto const cursor =
            make_owning_ptr(mongoac_database_list_collections(db, session, nullptr, nullptr), &mongoac_cursor_destroy);
         CHECK_FALSE(mongoac_cursor_next(cursor, nullptr)); // No collections in the database.
      }

      {
         auto const cursor =
            make_owning_ptr(mongoac_database_list_collections(db, session, nullptr, nullptr), &mongoac_cursor_destroy);
         CHECK_FALSE(mongoac_cursor_next(cursor, nullptr)); // No collections in the database.
      }

      assert_session_events(true);
   }
}
