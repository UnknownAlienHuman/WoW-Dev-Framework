---@meta _
C_Test = {}

---Inspect values.; No runtime body.
---
---[Documentation](https://warcraft.wiki.gg/wiki/API_C_Test.Inspect)
---@param enabled? boolean Default = false
---@param count? number Default = 0
---@param items? Enum.AccountData[]
---@param ... string values
---@return string? result
---@return Enum.AccountData kind
function C_Test.Inspect(enabled, count, items, ...) end

---@class Record
---@field items Enum.AccountData[]?

---@alias OnResult FunctionContainer|fun(count?: number)