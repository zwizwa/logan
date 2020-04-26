-module(lars).
-export([start_link/1, handle/2]).


start_link(Config) ->
    try
        case Config of
            #{ dev := Dev, type := Type } when
                  is_atom(Dev) and is_atom(Type) ->
                {ok, 
                 serv:start(
                   {handler,
                    fun() -> Config end,
                    fun ?MODULE:handle/2})}
        end
    catch _C:_E ->
            error
    end.

handle(restart, State) ->
    timer:send_after(2000, start),
    handle(stop, State);

handle(start, State = #{ port := _}) ->
    log:info("already started~n"),
    State;

handle(start, State = #{ spawn_port := SpawnPort, dev := Dev, type := Type }) ->
    %% Ask framework to spawn the port process.
    Port =
        SpawnPort(
          #{ opts => [{line,1024}, binary, use_stdio, exit_status],
             cmd  => "lars",
             args => [tools:format("~s",[Dev]),
                      tools:format("~s",[Type])]
           }),
    maps:put(port, Port, State);

handle(stop, State = #{port := Port}) ->
    port_close(Port),
    maps:remove(port, State);

handle(stop, State) ->
    log:info("already stopped~n"),
    State;
    

handle(Msg={_,dump}, State) ->
    obj:handle(Msg, State);

handle(Msg, State) ->
    log:info("unknown: ~p~n", [Msg]),
    State.


