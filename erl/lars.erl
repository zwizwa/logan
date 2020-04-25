-module(lars).
-export([start_link/1, handle/2]).
start_link(Config) ->
    try
        case Config of
            #{ dev := saleae } ->
                {ok, serv:start(
                       {handler,
                        fun() -> Config end,
                        fun ?MODULE:handle/2})}
        end
    catch _C:_E ->
            error
    end.

handle(Msg, State) ->
    obj:handle(Msg, State).

%% Split it in two parts: lars/examples contains scripts that can also
%% be used stand-alone.  The only convention that we need is that
%% closing of stdin needs to exit the application.
