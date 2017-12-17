# csv-aggregator
A tool to help aggregate csvs

## Tutorial
- Create a file `config.yaml`
```yaml
fields:
  - name: 'date'
    type: 'Date'
    format: '%Y/%m/%d'
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
    format: '%Y/%m/%d'
  - name: amount
    type: 'Number'
  - 'account'
  - 'desc'
  - 'desc2'
sort: 'date'
+filter:
+  - "if 'assets:chequing' then not 'transfer from savings'"
+  - if 'assets:savings' then not 'transfer from chequing'"
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

### Explanation
Filters run before csvs get merged into one. Every record that matches the filter,
will be accepted and passed through to the aggregated csv.

A quoted word `'chequing'` matches any field in the csv record.
eg. all the below match `'chequing'`
```csv
# "transfer to chequing account" matches
"2017/11/28",-2,"assets:savings","transfer","transfer to chequing account"
# "assets:chequing" matches
"2017/11/28",-2,"assets:chequing","transfer","transfer to savings account"
# "transfer to chequing account" matches
"2017/11/28",-2,"liabilities:credit","transfer","transfer to chequing account"
```

`not` reverses the result of the match:
eg. all the below matches `not 'chequing'`
```csv
# no fields match 'chequing'
"2017/11/28",-2,"assets:saving","interest","interest received"
# no fields match 'chequing'
"2017/11/28",-2,"liabilities:credit","best buy","buy something"
```

`if <condition> then <match>` will return the result of `<match>` if `<condition>`
matches:
eg. all the below matches `if 'assets:chequing' then 'transfer'`
```csv
# "assets:chequing" matches, so we check if "transfer" matches (it does)
"2017/11/28",-2,"assets:chequing","transfer","transfer to savings account"
# "assets:chequing" matches, so we check if "transfer" matches (it does)
"2017/11/30",-2,"assets:chequing","transfer","transfer to credit card"
```
