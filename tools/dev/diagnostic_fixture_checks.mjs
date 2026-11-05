/** Independent expectations for the user-reported diagnostic examples, never rendering rules. */
const lines=source=>source.split(/\r\n|[\r\n]/u);
function primarySpan(source) {
  let code;
  for(const line of lines(source)){
    const row=/^[ \t]*\d+ \| (.*)$/u.exec(line);
    if(row){code=row[1];continue;}
    const marker=/^[ \t]*\| ( *)(\^+)(?: |$)/u.exec(line);
    if(marker&&code!==undefined)return code.slice(marker[1].length,marker[1].length+marker[2].length);
  }
}

/** Inspect actual displayed text; full multiline span ownership is checked in Rust.
 * For example, diagnosticFixtureProblems('neg_cmp_float', renderedText) accepts the complete expression.
 */
export function diagnosticFixtureProblems(rule,source){
  const problems=[];
  const expected={neg_cmp_float:'!(x < limit)',manual_array_find:'for names',almost_swapped:'a = b;'}[rule];
  if(expected!==undefined&&primarySpan(source)!==expected)problems.push(`primary span: expected ${JSON.stringify(expected)}, received ${JSON.stringify(primarySpan(source))}`);
  if(rule==='manual_index_counter'){
    const rows=lines(source).flatMap(line=>{
      const row=/^[ \t]*(\d+) ([~+-]| ) (.*)$/u.exec(line);
      if(row)return[{number:Number(row[1]),mark:row[2],code:row[3].trim()}];
      const blank=/^[ \t]*(\d+)$/u.exec(line);
      return blank?[{number:Number(blank[1]),mark:' ',code:''}]:[];
    });
    const expectedRows=[[3,' '],[4,'-'],[5,' '],[6,' '],[6,'~'],[8,'-'],[9,' ']];
    if(JSON.stringify(rows.map(row=>[row.number,row.mark]))!==JSON.stringify(expectedRows))problems.push('fix rows are not source-ordered changes with neutral surrounding context');
    if(!rows[0]?.code.startsWith('show ::')||rows[1]?.code!=='n := 0;'||rows[2]?.code!==''||rows[3]?.code!=='for names {'||!rows[4]?.code.includes('it_index')||rows[5]?.code!=='n += 1;'||rows[6]?.code!=='}')problems.push('fix preview lost surrounding source or changed code');
  }
  return problems;
}
