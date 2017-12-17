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
```csv
"2017/11/30",-1,"assets:chequing","bill payment","credit card"
```
`csvs/b.csv`
```csv
"2017/11/28",-2,"assets:savings","transfer","transfer to chequing account"
```
`csvs/c.csv`
```csv
"2017/11/26",5,"liabilities:credit card","taco bell",""
```
- Run `csv-aggregator -c config.yaml 'csvs/*.csv'`

The output should be:
```csv
"2017/11/26",5,"liabilities:credit card","taco bell",""
"2017/11/28",-2,"assets:savings","transfer","transfer to chequing account"
"2017/11/30",-1,"assets:chequing","bill payment","credit card"
```

## Filters
This tool can also help you filter out CSV records. If you have a bunch of
transaction csvs for each bank account, you'll find that transfers between bank
accounts have a duplicated record: one in the sending, and one in the receiving.

Filters can help you remove one of the records before they get to the aggregated
csv.

### Basic tutorial
- Add this to your `config.yaml`
```diff
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
+filter:
+  - if 'chequing' in account then not 'transfer from savings'
+  - if 'savings' in account then not 'transfer from savings'
```
- Create a directory `csvs` with the following files:

`csvs/a.csv`
```csv
"2017/11/28",2,"assets:chequing","transfer","transfer from savings account"
"2017/11/30",-1,"assets:chequing","bill payment","credit card"
"2017/12/02",-1,"assets:chequing","transfer","transfer to savings account"
```
`csvs/b.csv`
```csv
"2017/11/28",-2,"assets:savings","transfer","transfer to chequing account"
"2017/11/30",0.5,"assets:savings","interest","interested received"
"2017/12/02",1,"assets:savings","transfer","transfer from chequing account"
```
- Run `csv-aggregator -c config.yaml 'csvs/*.csv'`

The output should be:
```
"2017/11/28",-2,"assets:savings","transfer","transfer to chequing account"
"2017/11/30",-1,"assets:chequing","bill payment","credit card"
"2017/11/30",0.5,"assets:savings","interest","interested received"
"2017/12/02",-1,"assets:chequing","transfer","transfer to savings account"
```
