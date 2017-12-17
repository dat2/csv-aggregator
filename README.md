# csv-aggregator
A tool to help aggregate csvs

## Tutorial
- Create a file `config.yaml`
```yaml
fields:
  - name: 'date'
    type: 'Date'
    format: 'YYYY/MM/DD'
  - name: amount
    type: 'Number'
  - 'account'
  - 'desc'
  - 'desc2'
sort: 'date'
```
- Create a directory `csvs` with the following files:

`csvs/a.csv`
```
"2017/11/30",-1,"assets:chequing","bill payment","credit card"
```
`csvs/b.csv`
```
"2017/11/28",2,"assets:savings","transfer","transfer to chequing account"
```
`csvs/c.csv`
```
"2017/11/26",5,"liabilities:credit card","taco bell",""
```
- Run `csv-aggregator -c config.yaml 'csvs/*.csv'`

The output should be:
```
"2017/11/26",5,"liabilities:credit card","taco bell",""
"2017/11/28",2,"assets:savings","transfer","transfer to chequing account"
"2017/11/30",-1,"assets:chequing","bill payment","credit card"
```
