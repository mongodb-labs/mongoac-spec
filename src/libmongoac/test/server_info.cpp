#include <mongoac/server_info.h>

#include <mongoac/bson.h>
#include <mongoac/string.h>

//

#include <catch2/catch_test_macros.hpp>

TEST_CASE("null info", "[mongoac][server_info]")
{
   SECTION("get_type")
   {
      CHECK(mongoac_server_info_get_type(nullptr) == MONGOAC_SERVER_TYPE_UNKNOWN);
   }

   SECTION("get_host")
   {
      mongoac_string_view_t const host = mongoac_server_info_get_host(nullptr);
      CHECK(host.data == nullptr);
      CHECK(host.len == 0);
   }

   SECTION("get_port")
   {
      CHECK(mongoac_server_info_get_port(nullptr) == 0);
   }

   SECTION("get_average_round_trip_time")
   {
      CHECK(mongoac_server_info_average_round_trip_time_has_value(nullptr) == false);
      CHECK(mongoac_server_info_get_average_round_trip_time_secs(nullptr) == 0);
      CHECK(mongoac_server_info_get_average_round_trip_time_nanos(nullptr) == 0);
   }

   SECTION("get_last_update_time")
   {
      CHECK(mongoac_server_info_last_update_time_has_value(nullptr) == false);
      CHECK(mongoac_server_info_get_last_update_time(nullptr) == 0);
   }

   SECTION("get_max_wire_version")
   {
      CHECK(mongoac_server_info_max_wire_version_has_value(nullptr) == false);
      CHECK(mongoac_server_info_get_max_wire_version(nullptr) == 0);
   }

   SECTION("get_min_wire_version")
   {
      CHECK(mongoac_server_info_min_wire_version_has_value(nullptr) == false);
      CHECK(mongoac_server_info_get_min_wire_version(nullptr) == 0);
   }

   SECTION("get_replica_set_name")
   {
      mongoac_string_view_t const name = mongoac_server_info_get_replica_set_name(nullptr);
      CHECK(name.data == nullptr);
      CHECK(name.len == 0);
   }

   SECTION("get_replica_set_version")
   {
      CHECK(mongoac_server_info_replica_set_version_has_value(nullptr) == false);
      CHECK(mongoac_server_info_get_replica_set_version(nullptr) == 0);
   }

   SECTION("get_tags")
   {
      mongoac_bson_t const tags = mongoac_server_info_get_tags(nullptr);
      CHECK(tags.data == nullptr);
      CHECK(tags.len == 0);
   }

   SECTION("has_error")
   {
      CHECK(mongoac_server_info_has_error(nullptr) == false);
   }
}
