set (input_vars
  src_dir
  bin_dir
  prefix
  includedir
  libdir
  output_name
  soname
  version
  is_static
  bson_req_ver
)

foreach (var ${input_vars})
  if (NOT DEFINED "${var}")
    message (FATAL_ERROR "${var} was not set!")
  endif ()
endforeach ()

if (1)
  set (requires "")

  if (is_static)
    list (APPEND requires "bson2-static >= ${bson_req_ver}")
  else ()
    list (APPEND requires "bson2 >= ${bson_req_ver}")
  endif ()

  list (JOIN requires ", " requires)
endif ()

if (1)
  set (cflags "")

  if (is_static)
    list (APPEND cflags "-DBSON_STATIC")
  endif ()

  list (APPEND cflags "-I\${includedir}")
  list (JOIN cflags " " cflags)
endif ()

if (is_static)
  set (pkgname "libmongoac-static")
  set (libs "\${libdir}/lib${soname}.a")
  set (pc_output "${bin_dir}/${output_name}-static.pc")
else ()
  set (pkgname "libmongoac")
  set (libs "-L\${libdir} -l${soname}")
  set (pc_output "${bin_dir}/${output_name}.pc")
endif ()

configure_file (
  ${src_dir}/mongoac.pc.in
  ${pc_output}
  @ONLY
)
