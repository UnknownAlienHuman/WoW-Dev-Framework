StateCoreAccountDB = {}
StateCoreCharacterDB = {}

-- Direct literal read and write on the account root.
local function read_account()
  return StateCoreAccountDB.profile and StateCoreAccountDB.profile.name
end

local function write_account()
  StateCoreAccountDB.profile = { name = "fixture" }
  StateCoreAccountDB.settings = {}
  StateCoreAccountDB.settings.volume = 10
end

-- Direct literal write on the character root.
local function write_character()
  StateCoreCharacterDB.gold = 5
end

-- Lexical alias reads: the alias carries the exact root identity.
local function alias_read()
  local account = StateCoreAccountDB
  local character = StateCoreCharacterDB
  return account.profile, character.gold
end

-- Shadowed local: the local binding must not create an edge. This is the
-- honest negative case: no fabricated state path.
local function shadowed_local()
  local StateCoreAccountDB = {}
  return StateCoreAccountDB.profile
end

-- Dynamic key: the resolved prefix is retained but the whole path stays
-- unresolved, so no exact path entity is created.
local function dynamic_key(key)
  StateCoreAccountDB[key] = 1
end
