#include <test_util/bson.hh>

#include <bson/bson.h>

#include <cstdint>
#include <cstring>

namespace mongoac
{
namespace test_util
{

bson_t *
bson_from_json(const char *json)
{
   REQUIRE(json != nullptr);

   bson_error_t error = {};

   if (auto const ret = bson_new_from_json(reinterpret_cast<std::uint8_t const *>(json), -1, &error)) {
      return ret;
   }

   FAIL(error.message);
}

bool
bson_array_contains_string(const bson_t *array, const char *str)
{
   bson_iter_t iter = {};

   REQUIRE(array != nullptr);

   if (!bson_iter_init(&iter, array)) {
      return false;
   }

   while (bson_iter_next(&iter)) {
      if (!BSON_ITER_HOLDS_UTF8(&iter)) {
         continue;
      }

      auto const value = bson_iter_utf8(&iter, nullptr);

      if (value && std::strcmp(value, str) == 0) {
         return true;
      }
   }

   return false;
}

} // namespace test_util
} // namespace mongoac
