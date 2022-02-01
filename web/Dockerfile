FROM python:3

RUN mkdir -p /app

WORKDIR /app

ENV FLASK_APP=app.py
ENV FLASK_RUN_HOST=0.0.0.0

COPY requirements.txt .

RUN python -m pip install -r requirements.txt

COPY . .

EXPOSE 5000

CMD ["flask", "run"]
