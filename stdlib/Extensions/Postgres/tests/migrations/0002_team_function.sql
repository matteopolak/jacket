create function mig_team_count() returns int8 language plpgsql as $body$
begin
    return (select count(*) from mig_teams); -- ; inside a dollar quote
end;
$body$;
