#include <test_util/bson.hpp>

#include <bson/bson.h>

namespace mongoac
{
namespace test_util
{

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
