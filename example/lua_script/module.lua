---
--- Generated by EmmyLua(https://github.com/EmmyLua)
--- Created by bytedance.
--- DateTime: 2023/9/26 4:37 PM
---

module = {name="show_module"}

module.show_req = function(req)
    for i, v in pairs(req) do
        print("show--->",i,v)
    end
end

return module