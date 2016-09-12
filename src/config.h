// Copyright 2015 Keegan McAllister.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// See `LICENSE` in this repository.

#define SHM_ENV_VAR         "__AFL_SHM_ID"

#define MAP_SIZE_POW2       16
#define MAP_SIZE            (1 << MAP_SIZE_POW2)

#define FORKSRV_FD          198

#define DEFER_ENV_VAR       "__AFL_DEFER_FORKSRV"
#define PERSIST_ENV_VAR     "__AFL_PERSISTENT"
