module = {name="show_module"}

module.show_req = function(req)
    for i, v in pairs(req) do
        print("show--->",i,v)
    end
end

return module