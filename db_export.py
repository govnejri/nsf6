#!/usr/bin/env python3
import pg8000.native
import sys
from datetime import datetime

def connect_and_export():
    # Параметры подключения к базе данных
    host = "127.0.0.1"
    port = 5432
    database = "nsf6_db"
    user = "nsf6"
    password = "yourpassword"
    
    try:
        # Подключение к базе данных
        print("Подключение к базе данных...")
        conn = pg8000.native.Connection(
            host=host,
            port=port,
            database=database,
            user=user,
            password=password
        )
        
        # Выполнение запроса
        print("Выполнение запроса SELECT * FROM points...")
        rows = conn.run("SELECT * FROM points;")
        
        # Получение информации о колонках (для pg8000 нужно отдельный запрос)
        columns_info = conn.run("""
            SELECT column_name 
            FROM information_schema.columns 
            WHERE table_name = 'points' 
            ORDER BY ordinal_position;
        """)
        column_names = [col[0] for col in columns_info]
        
        # Создание имени файла с временной меткой
        timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
        filename = f"points_export_{timestamp}.txt"
        
        # Сохранение в файл
        print(f"Сохранение данных в файл {filename}...")
        with open(filename, 'w', encoding='utf-8') as f:
            # Записываем заголовок с названиями колонок
            f.write("Экспорт данных из таблицы points\n")
            f.write(f"Дата и время экспорта: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n")
            f.write("=" * 50 + "\n\n")
            
            # Записываем названия колонок
            f.write("Колонки: " + " | ".join(column_names) + "\n")
            f.write("-" * 50 + "\n")
            
            # Записываем данные
            if rows:
                for row in rows:
                    # Преобразуем каждое значение в строку
                    row_str = " | ".join(str(value) if value is not None else "NULL" for value in row)
                    f.write(row_str + "\n")
                
                f.write(f"\nВсего записей: {len(rows)}\n")
            else:
                f.write("Данные в таблице отсутствуют.\n")
        
        print(f"Экспорт завершен успешно! Сохранено {len(rows)} записей в файл {filename}")
        
    except Exception as e:
        print(f"Ошибка: {e}")
        sys.exit(1)
    finally:
        # Закрытие соединения
        if 'conn' in locals():
            conn.close()
        print("Соединение с базой данных закрыто.")

if __name__ == "__main__":
    connect_and_export()
